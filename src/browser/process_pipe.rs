use crate::browser::default_executable;
use crate::browser::launch_options::{LaunchOptions, DEFAULT_ARGS};

use failure::{format_err, Fallible};
use log::{info, trace, warn};
use std::path::PathBuf;
use tempfile::TempDir;

//Todo: Send proper error if chrome binary not found

#[cfg(feature = "fetch")]
use super::fetcher::{Fetcher, FetcherOptions};

#[cfg(unix)]
use nix::{
    fcntl::{open, OFlag},
    sys::{
        signal::{kill, SIGKILL},
        stat::Mode,
        wait::{waitpid, WaitStatus},
    },
    unistd::{close, dup2, execvp, fork, ForkResult, Pid},
};
#[cfg(unix)]
use std::ffi::{CStr, CString};
#[cfg(unix)]
use std::net::Shutdown;
#[cfg(unix)]
use std::os::unix::{
    io::{AsRawFd, RawFd},
    net::UnixStream,
};
#[cfg(unix)]
use std::process::abort;

#[cfg(windows)]
use modular_bitfield::prelude::*;
#[cfg(windows)]
use std::ffi::{OsStr, OsString};
#[cfg(windows)]
use std::fs::{File, OpenOptions};
#[cfg(windows)]
use std::os::windows::{
    ffi::OsStrExt,
    io::{AsRawHandle, FromRawHandle, IntoRawHandle, RawHandle},
};
#[cfg(windows)]
use winapi::{
    _core::{iter, mem, ptr},
    ctypes::{c_uchar, c_uint, c_void},
    shared::minwindef::{BOOL, DWORD},
    um::{
        errhandlingapi::GetLastError,
        fileapi::{CreateFileW, OPEN_EXISTING},
        handleapi::{CloseHandle, INVALID_HANDLE_VALUE},
        minwinbase::{LPSECURITY_ATTRIBUTES, SECURITY_ATTRIBUTES},
        namedpipeapi::{ConnectNamedPipe, CreateNamedPipeW},
        processthreadsapi::{CreateProcessW, TerminateProcess, PROCESS_INFORMATION, STARTUPINFOW},
        synchapi::WaitForSingleObject,
        winbase::{
            CREATE_NEW_PROCESS_GROUP, CREATE_UNICODE_ENVIRONMENT, DETACHED_PROCESS,
            FILE_FLAG_FIRST_PIPE_INSTANCE, FILE_FLAG_OVERLAPPED, PIPE_ACCESS_DUPLEX,
            PIPE_READMODE_BYTE, PIPE_TYPE_BYTE, PIPE_WAIT, STARTF_USESHOWWINDOW,
            STARTF_USESTDHANDLES, INFINITE,
        },
        winnt::{FILE_READ_ATTRIBUTES, GENERIC_READ, GENERIC_WRITE},
        winreg::HKEY_LOCAL_MACHINE,
    },
};
#[cfg(windows)]
use winreg::RegKey;

pub struct Process {
    pub child_process: Child,
    user_data_dir: Option<TempDir>,
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

        // NOTE: picking random data dir so that each a new browser instance is launched
        // (see man google-chrome)
        let user_data_dir = ::tempfile::Builder::new().prefix("rhc-profile").tempdir()?;

        let process: Child = Self::start_process(&launch_options, &user_data_dir)?;
        info!("Started Chrome. PID: {}", process.id());

        Ok(Self {
            child_process: process,
            user_data_dir: Some(user_data_dir),
        })
    }
    fn start_process(launch_options: &LaunchOptions, user_data_dir: &TempDir) -> Fallible<Child> {
        let window_size_option = if let Some((width, height)) = launch_options.window_size {
            format!("--window-size={},{}", width, height)
        } else {
            String::from("")
        };

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

        if !window_size_option.is_empty() && launch_options.headless {
            args.extend(&[window_size_option.as_str()]);
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

        if launch_options.headless {
            // Headless mode won't run if it doesn't have a page to load for some reason (windows)
            args.extend(&["--headless", "chrome://version"]);
        }

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

#[cfg(unix)]
impl Drop for Process {
    fn drop(&mut self) {
        let _i = self.child_process.input.shutdown(Shutdown::Both);
        let _o = self.child_process.output.shutdown(Shutdown::Both);

        info!("Killing Chrome. PID: {}", self.child_process.id());
        self.child_process
            .kill()
            .and_then(|_| self.child_process.wait())
            .ok();
        if let Some(dir) = self.user_data_dir.take() {
            if let Err(e) = dir.close() {
                warn!("Failed to close temp directory: {}", e);
            }
        }
    }
}

#[cfg(windows)]
impl Drop for Process {
    fn drop(&mut self) {
        info!("Killing Chrome. PID: {}", self.child_process.id());
        self.child_process
            .kill()
            .ok();
        if let Some(dir) = self.user_data_dir.take() {
            if let Err(e) = dir.close() {
                warn!("Failed to close temp directory: {}", e);
            }
        }
    }
}

#[cfg(windows)]
pub(crate) fn get_chrome_path_from_registry() -> Option<std::path::PathBuf> {
    RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\App Paths\\chrome.exe")
        .and_then(|key| key.get_value::<String, _>(""))
        .map(std::path::PathBuf::from)
        .ok()
}

// Todo: add environment variables to child process
#[cfg(unix)]
pub fn spawn(path: &PathBuf, args: Vec<&str>) -> Fallible<Child> {
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

#[cfg(windows)]
pub fn spawn(path: &PathBuf, args: Vec<&str>) -> Fallible<Child> {
    let (input_pipe1, input_pipe2) = create_pipe()?;
    let (output_pipe1, output_pipe2) = create_pipe()?;

    let pipes = pipe_factory(input_pipe2.as_raw_handle(), output_pipe2.as_raw_handle())?;

    let app_name: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(iter::once(0u16))
        .collect();

    let mut command_line: Vec<u16> = path
        .file_name()
        .map(|f| format!("{} {}", f.to_str().unwrap(), args.join(" ")))
        .map(|c| {
            OsStr::new(&c)
                .encode_wide()
                .chain(iter::once(0u16))
                .collect::<Vec<u16>>()
        })
        .ok_or_else(|| format_err!("Chrome path required"))?;

    let mut startup: STARTUPINFOW = unsafe { mem::zeroed() };
    startup.cb = mem::size_of::<STARTUPINFOW>() as DWORD;
    startup.hStdInput = pipes.get_stdin() as *mut c_void;
    startup.hStdOutput = pipes.get_stdout() as *mut c_void;
    startup.hStdError = pipes.get_stderr() as *mut c_void;
    startup.dwFlags = STARTF_USESTDHANDLES | STARTF_USESHOWWINDOW;

    let mut pipes_bytes = pipes.to_bytes().to_owned();

    startup.cbReserved2 = pipes_bytes.len() as u16;
    startup.lpReserved2 = pipes_bytes.as_mut_ptr();

    let mut pinfo: PROCESS_INFORMATION = unsafe { mem::zeroed() };

    let process_flags = CREATE_UNICODE_ENVIRONMENT | DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP;

    // Todo: Environment
    let env = ptr::null_mut();

    let _ret: BOOL = unsafe {
        CreateProcessW(
            app_name.as_ptr(),
            command_line.as_mut_ptr(),
            ptr::null_mut(),
            ptr::null_mut(),
            true as BOOL,
            process_flags,
            env,
            ptr::null_mut(),
            &mut startup,
            &mut pinfo,
        )
    };
    let err = unsafe { CloseHandle(pinfo.hThread) };

    if err == 0 {
        Err(std::io::Error::last_os_error().into())
    } else {
        Ok(Child {
            pid: pinfo.dwProcessId,
            input: input_pipe1,
            output: output_pipe1,
            handle: Handle(pinfo.hProcess),
        })
    }
}

#[cfg(windows)]
pub struct Child {
    pub pid: u32,
    pub input: File,
    pub output: File,
    handle: Handle,
}

#[cfg(windows)]
impl Child {
    pub fn id(&self) -> u32 {
        self.pid
    }
    pub fn kill(&mut self) -> Fallible<()> {
        unsafe {
            TerminateProcess(self.handle.as_raw_handle(), 1);
            WaitForSingleObject(self.handle.as_raw_handle(), INFINITE);
            CloseHandle(self.handle.as_raw_handle());
        };
        Ok(())
    }
}

#[cfg(windows)]
fn create_pipe() -> Fallible<(File, File)> {
    let mut os_str: OsString = OsStr::new(r#"\\.\pipe\headless-chrome-"#).into();
    os_str.push(rand::random::<u16>().to_string());
    os_str.push("\x00");
    let u16_slice = os_str.encode_wide().collect::<Vec<u16>>();

    let access_flags = PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED | FILE_FLAG_FIRST_PIPE_INSTANCE;

    let server_handle: RawHandle = unsafe {
        CreateNamedPipeW(
            u16_slice.as_ptr(),
            access_flags,
            PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
            1,
            65536,
            65536,
            0,
            ptr::null_mut(),
        )
    };

    let mut attributes = SECURITY_ATTRIBUTES {
        nLength: mem::size_of::<SECURITY_ATTRIBUTES>() as DWORD,
        lpSecurityDescriptor: ptr::null_mut(),
        bInheritHandle: true as BOOL,
    };

    let child_handle: RawHandle = unsafe {
        CreateFileW(
            u16_slice.as_ptr(),
            GENERIC_READ | GENERIC_WRITE | FILE_READ_ATTRIBUTES,
            0,
            &mut attributes as LPSECURITY_ATTRIBUTES,
            OPEN_EXISTING,
            0,
            ptr::null_mut(),
        )
    };

    if server_handle != INVALID_HANDLE_VALUE && child_handle != INVALID_HANDLE_VALUE {
        let ret = unsafe { ConnectNamedPipe(server_handle, ptr::null_mut()) != 0 };
        if !ret {
            let err = unsafe { GetLastError() };
            if err != 535 {
                return Err(failure::err_msg("Pipe error"));
            }
        }
        let server = unsafe { File::from_raw_handle(server_handle) };
        let client = unsafe { File::from_raw_handle(child_handle) };
        Ok((server, client))
    } else {
        Err(std::io::Error::last_os_error().into())
    }
}

/*
The buffer has the following layout:
*   int number_of_fds
*   unsigned char crt_flags[number_of_fds]
*   HANDLE os_handle[number_of_fds]
*/
#[cfg(windows)]
#[bitfield]
#[derive(Debug, PartialEq, Eq)]
pub struct Pipes {
    fd_count: B32,
    crt_flag_stdin: B8,
    crt_flag_stdout: B8,
    crt_flag_stderr: B8,
    crt_flag_childin: B8,
    crt_flag_childout: B8,
    stdin: B32,
    stdout: B32,
    stderr: B32,
    childin: B32,
    childout: B32,
}

#[cfg(windows)]
fn pipe_factory(child_input: RawHandle, child_output: RawHandle) -> Fallible<Pipes> {
    let mut pipes = Pipes::new();

    let fd_count: c_uint = 5;
    let flag: c_uchar = 0x01 | 0x40;

    pipes.set_fd_count(fd_count);
    pipes.set_crt_flag_stdin(flag);
    pipes.set_crt_flag_stdout(flag);
    pipes.set_crt_flag_stderr(flag);
    pipes.set_crt_flag_childin(flag);
    pipes.set_crt_flag_childout(flag);

    let pipe_null = OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .create(true)
        .open(r#"\\.\NUL"#)?;

    pipes.set_stdin(pipe_null.try_clone()?.into_raw_handle() as u32);
    pipes.set_stdout(pipe_null.try_clone()?.into_raw_handle() as u32);
    pipes.set_stderr(pipe_null.try_clone()?.into_raw_handle() as u32);
    pipes.set_childin(child_input as u32);
    pipes.set_childout(child_output as u32);

    Ok(pipes)
}

#[cfg(windows)]
#[derive(Debug)]
pub struct Handle(RawHandle);

#[cfg(windows)]
unsafe impl Send for Handle {}

#[cfg(windows)]
impl AsRawHandle for Handle {
    fn as_raw_handle(&self) -> RawHandle {
        self.0
    }
}

#[cfg(windows)]
impl FromRawHandle for Handle {
    unsafe fn from_raw_handle(handle: RawHandle) -> Handle {
        Handle(handle)
    }
}
