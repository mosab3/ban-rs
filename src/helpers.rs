use std::path::{Path, PathBuf};
use std::process::Command;
use sysinfo::System;

/// Check if the system is Red Hat based
pub fn is_red_hat_based() -> bool {

    // Check for the existence of the file `/etc/redhat-release`
    return Path::new("/etc/redhat-release").exists()
}

/// Check if the service is running
pub fn check_service_status(service: String) -> bool {
    let output = Command::new("systemctl")
        .arg("status")
        .arg(service)
        .output()
        .expect("Failed to execute command");

    return output.status.success();
}

/// Calculate the optimal channel capacity based on available memory and message size
pub fn calculate_optimal_channel_capacity() -> usize {
    const MEMORY_FRACTION: f64 = 0.05; // Use 5% of available memory for the channel buffer
    const AVERAGE_MESSAGE_SIZE: usize = 512; // Estimate average message size in bytes

    // Create a new system instance and refresh the memory information
    let mut system = System::new_all();
    system.refresh_all();

    // Get available memory in bytes
    let available_memory = system.available_memory();

    // Estimate channel capacity based on the memory fraction percentage
    let memory_for_channel = available_memory as f64 * MEMORY_FRACTION;
    let channel_capacity = memory_for_channel / AVERAGE_MESSAGE_SIZE as f64;

    // Return the capacity as usize
    channel_capacity as usize
}

/// Check if the current user has root privileges
pub fn is_root() -> bool {
    let uid: u32 = unsafe {
        libc::geteuid()
    };
    
    uid == 0
}

// fn get_active_http_service() -> Result<HttpType, String> {
//     // Check the active http service


//     let services = vec!["apache2", "nginx"];

//     let mut active_service = Err("No active service found");

//     for service in services {
//         // Validate the first active service from the list of `services`
//         if check_service_status(service) {
//             active_service = match service {
//                 "apache2" => Ok(HttpType::APACHE),
//                 "nginx" => Ok(HttpType::NGINX),
//                 _ => todo!(),
//             };
//             break;
//         }
//     };
    
//     return Ok(active_service?);
// }

// pub fn get_http_logfile_path() -> PathBuf {
//     // Get the active http service logging file path

//     let active_service = get_active_http_service().expect("Failed to get active HTTP service");
//     let is_red_hat = is_red_hat_based();

//     match active_service {
//         HttpType::APACHE => {
//             if is_red_hat {
//                 return Path::new("/var/log/httpd/access_log").to_path_buf();
//             } else {
//                 return Path::new("/var/log/apache2/access.log").to_path_buf();
//             }
//         },
//         HttpType::NGINX => {
//             Path::new("/var/log/nginx/access.log").to_path_buf()
//         },
//         _ => panic!("issue with getting http logfile path")
//     }
// }
