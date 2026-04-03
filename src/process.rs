use thiserror::Error;
use std::path::PathBuf;

#[derive(Error, Debug)]
pub enum ProcessError {
    #[error("Process not found: {0}")]
    ProcessNotFound(u32),
    #[error("Failed to attach to process: {0}")]
    AttachFailed(String),
    #[error("Failed to read memory: {0}")]
    ReadMemoryFailed(String),
    #[error("Failed to write memory: {0}")]
    WriteMemoryFailed(String),
    #[error("Injection failed: {0}")]
    InjectionFailed(String),
    #[error("Python interpreter not found")]
    PythonNotFound,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Feature not supported on this platform")]
    NotSupported,
}

#[cfg(windows)]
pub fn attach_to_process(pid: u32) -> Result<(), ProcessError> {
    use windows::Win32::Foundation::*;
    use windows::Win32::System::Threading::*;

    unsafe {
        let handle = OpenProcess(
            PROCESS_ALL_ACCESS,
            false,
            pid,
        ).map_err(|e| ProcessError::AttachFailed(e.to_string()))?;

        tracing::info!("Opened process {} with handle {:?}", pid, handle);
        Ok(())
    }
}

#[cfg(not(windows))]
pub fn attach_to_process(pid: u32) -> Result<(), ProcessError> {
    tracing::warn!("Process attach not supported on this platform");
    Ok(())
}

#[cfg(windows)]
pub fn detach_from_process() -> Result<(), ProcessError> {
    Ok(())
}

#[cfg(not(windows))]
pub fn detach_from_process() -> Result<(), ProcessError> {
    Ok(())
}

#[cfg(windows)]
pub fn read_process_memory(pid: u32, address: usize, size: usize) -> Result<Vec<u8>, ProcessError> {
    use windows::Win32::Foundation::*;
    use windows::Win32::System::Threading::*;

    unsafe {
        let handle = OpenProcess(
            PROCESS_VM_READ,
            false,
            pid,
        ).map_err(|e| ProcessError::AttachFailed(e.to_string()))?;

        let mut buffer = vec![0u8; size];
        let mut bytes_read = 0usize;

        let result = windows::Win32::System::Memory::ReadProcessMemory(
            handle,
            address as *const std::ffi::c_void,
            buffer.as_mut_ptr() as *mut std::ffi::c_void,
            size,
            Some(&mut bytes_read),
        );

        match result {
            Ok(_) => {
                buffer.truncate(bytes_read);
                Ok(buffer)
            }
            Err(e) => Err(ProcessError::ReadMemoryFailed(e.to_string())),
        }
    }
}

#[cfg(not(windows))]
pub fn read_process_memory(pid: u32, address: usize, size: usize) -> Result<Vec<u8>, ProcessError> {
    use std::fs;

    let path = format!("/proc/{}/mem", pid);
    let mut file = fs::File::open(&path)?;
    
    use std::io::Seek;
    file.seek(std::io::SeekFrom::Start(address as u64))?;
    
    let mut buffer = vec![0u8; size];
    let bytes_read = std::io::Read::read(&mut file, &mut buffer)?;
    buffer.truncate(bytes_read);
    
    Ok(buffer)
}

#[cfg(windows)]
pub fn write_process_memory(pid: u32, address: usize, data: &[u8]) -> Result<usize, ProcessError> {
    use windows::Win32::Foundation::*;
    use windows::Win32::System::Threading::*;

    unsafe {
        let handle = OpenProcess(
            PROCESS_VM_WRITE | PROCESS_VM_OPERATION,
            false,
            pid,
        ).map_err(|e| ProcessError::AttachFailed(e.to_string()))?;

        let mut bytes_written = 0usize;

        let result = windows::Win32::System::Memory::WriteProcessMemory(
            handle,
            address as *mut std::ffi::c_void,
            data.as_ptr() as *const std::ffi::c_void,
            data.len(),
            Some(&mut bytes_written),
        );

        match result {
            Ok(_) => Ok(bytes_written),
            Err(e) => Err(ProcessError::WriteMemoryFailed(e.to_string())),
        }
    }
}

#[cfg(not(windows))]
pub fn write_process_memory(pid: u32, address: usize, data: &[u8]) -> Result<usize, ProcessError> {
    use std::fs;

    let path = format!("/proc/{}/mem", pid);
    let mut file = fs::File::open(&path)?;
    
    use std::io::Seek;
    file.seek(std::io::SeekFrom::Start(address as u64))?;
    
    Ok(std::io::Write::write(&mut file, data)?)
}

#[cfg(windows)]
pub fn find_python_interpreter(_pid: u32) -> Result<PathBuf, ProcessError> {
    let python_paths = vec![
        "C:\\Python27\\python.exe",
        "C:\\Python2\\python.exe",
        "C:\\Program Files\\Python27\\python.exe",
    ];

    for path in python_paths {
        let p = PathBuf::from(path);
        if p.exists() {
            return Ok(p);
        }
    }

    Err(ProcessError::PythonNotFound)
}

#[cfg(not(windows))]
pub fn find_python_interpreter(_pid: u32) -> Result<PathBuf, ProcessError> {
    let python_paths = vec![
        "/usr/bin/python",
        "/usr/bin/python2",
        "/usr/bin/python2.7",
    ];

    for path in python_paths {
        let p = PathBuf::from(path);
        if p.exists() {
            return Ok(p);
        }
    }

    Err(ProcessError::PythonNotFound)
}

pub struct DebugSession {
    pub pid: u32,
    pub python_path: Option<PathBuf>,
}

impl DebugSession {
    pub fn new(pid: u32) -> Result<Self, ProcessError> {
        attach_to_process(pid)?;
        let python_path = find_python_interpreter(pid).ok();
        Ok(Self { pid, python_path })
    }

    pub fn detach(&self) -> Result<(), ProcessError> {
        detach_from_process()
    }
}