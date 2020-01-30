use crate::browser::default_executable;
use crate::browser::launch_options::{LaunchOptions, DEFAULT_ARGS};

#[cfg(unix)]
use nix::{
    fcntl::{open, OFlag},
    sys::{
        signal::{kill, SIGKILL},
        stat::Mode,
        wait::{waitpid, WaitStatus}
    },
    unistd::{Pid, close, dup2, execvp, fork, ForkResult},
};

#[cfg(unix)]
use std::os::unix::{
    io::{AsRawFd, RawFd},
    net::UnixStream,
};

use std::ffi::{CStr, CString};
use std::net::Shutdown;

use failure::{format_err, Fallible};
use log::{info, trace};
use std::path::PathBuf;
use std::process::{abort};

#[cfg(feature = "fetch")]
use super::fetcher::{Fetcher, FetcherOptions};

pub struct Process {
    pub child_process: Child,
}

impl Process {
    pub fn new(mut launch_options: LaunchOptions) -> Fallible<Self> {
        if launch_options.path.is_none() {
            #[cfg(feature = "fetch")]
            {
                let fetch = Fetcher::new(launch_options.fetcher_options.clone())?;
                launch_options.path = Some(fetch.fetch()?);
            }
            #[cfg(not(feature = "fetch"))]
            {
                launch_options.path = Some(default_executable().map_err(|e| format_err!("{}", e))?);
            }
        }

        let process: Child = Self::start_process(&launch_options)?;
        info!("Started Chrome. PID: {}", process.id());

        Ok(Self {
            child_process: process,
        })
    }
    fn start_process(launch_options: &LaunchOptions) -> Fallible<Child> {
        let window_size_option = if let Some((width, height)) = launch_options.window_size {
            format!("--window-size={},{}", width, height)
        } else {
            String::from("")
        };

        // NOTE: picking random data dir so that each a new browser instance is launched
        // (see man google-chrome)
        let user_data_dir = ::tempfile::Builder::new()
            .prefix("rust-headless-chrome-profile")
            .tempdir()?;
        let data_dir_option = format!("--user-data-dir={}", user_data_dir.path().to_str().unwrap());

        trace!("Chrome will have profile: {}", data_dir_option);

        let mut args = vec![
            "--remote-debugging-pipe",
            "--disable-gpu",
            "--enable-logging",
            "--verbose",
            "--log-level=0",
            "--no-first-run",
            "--disable-audio-output",
            data_dir_option.as_str(),
        ];

        args.extend(&DEFAULT_ARGS);

        if !launch_options.args.is_empty() {
            let extra_args: Vec<&str> = launch_options
                .args
                .iter()
                .map(|a| a.to_str().unwrap())
                .collect();
            args.extend(extra_args);
        }

        if !window_size_option.is_empty() {
            args.extend(&[window_size_option.as_str()]);
        }

        if launch_options.headless {
            args.extend(&["--headless"]);
        }

        if !launch_options.sandbox {
            args.extend(&["--no-sandbox", "--disable-setuid-sandbox"]);
        }

        let extension_args: Vec<String> = launch_options
            .extensions
            .iter()
            .map(|e| format!("--load-extension={}", e.to_str().unwrap()))
            .collect();

        args.extend(extension_args.iter().map(String::as_str));

        let path = launch_options
            .path
            .as_ref()
            .ok_or_else(|| format_err!("Chrome path required"))?;

        info!("Launching Chrome binary at {:?}", &path);
        spawn(&path, args)
    }
    pub fn get_id(&self) -> u32 {
        self.child_process.id()
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        let _i = self.child_process.input.shutdown(Shutdown::Both);
        let _o = self.child_process.output.shutdown(Shutdown::Both);

        info!("Killing Chrome. PID: {}", self.child_process.id());
        self.child_process
            .kill()
            .and_then(|_| self.child_process.wait())
            .ok();
    }
}

// Todo: add environment variables to child process
#[cfg(unix)]
pub fn spawn(
    path: &PathBuf,
    args: Vec<&str>,
) -> Fallible<Child> {
    let (input_socket1, input_socket2) = UnixStream::pair()?;
    let (output_socket1, output_socket2) = UnixStream::pair()?;

    let child_pid: Pid;

    // Todo: Create mutex to stop other threads from creating fd's until process has finished spawning
    /*
    unsafe {
        let s = env_lock();
    }
    */

    match fork() {
        Ok(ForkResult::Parent { child, .. }) => {
            close(input_socket2.as_raw_fd())?;
            close(output_socket2.as_raw_fd())?;
            child_pid = child;
        }
        Ok(ForkResult::Child) => {
            close(input_socket1.as_raw_fd())?;
            close(output_socket1.as_raw_fd())?;

            for stdio in 0..3 {
                let fd: RawFd =
                    open("/dev/null", OFlag::O_RDWR, Mode::S_IRWXU).expect("Unable to set stdio");
                dup2(fd, stdio).expect("Unable to set stdio");
            }

            dup2(input_socket2.as_raw_fd(), 3).expect("Unable to set stdio");
            dup2(output_socket2.as_raw_fd(), 4).expect("Unable to set stdio");

            let path = path
                .to_str()
                .map(|p| CString::new(p).expect("Unable to create CString"))
                .ok_or_else(|| format_err!("Chrome path required"))?;

            let args_vec: Vec<_> = args
                .iter()
                .map(|s| CString::new(s.as_bytes()).unwrap())
                .collect();
            let args_cstr: Vec<&CStr> = args_vec.iter().map(|c| c.as_c_str()).collect();

            let mut path_vec = vec![path.as_c_str()];

            path_vec.extend(args_cstr);

            let _res = execvp(path.as_c_str(), &path_vec)?;
            abort()
        }
        Err(_) => abort(),
    }

    Ok(Child {
        pid: child_pid,
        input: input_socket1,
        output: output_socket1,
        status: None,
    })
}

#[cfg(unix)]
pub struct Child {
    pid: Pid,
    pub input: UnixStream,
    pub output: UnixStream,
    status: Option<WaitStatus>,
}

#[cfg(unix)]
impl Child {
    pub fn id(&self) -> u32 {
        self.pid.as_raw() as u32
    }
    pub fn kill(&mut self) -> Fallible<()> {
        // If we've already waited on this process then the pid can be recycled
        // and used for another process, and we probably shouldn't be killing
        // random processes, so just return an error.
        if self.status.is_some() {
            Err(format_err!(
                "invalid argument: can't kill an exited process"
            ))
        } else {
            kill(self.pid, SIGKILL)?;
            Ok(())
        }
    }
    pub fn wait(&mut self) -> Fallible<WaitStatus> {
        if let Some(status) = self.status {
            return Ok(status);
        }
        let status = waitpid(self.pid, None)?;
        self.status = Some(status);
        Ok(status)
    }
}

