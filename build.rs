use std::{env, process::Command};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("protos/teleop.proto")?;

    let ros_distro = env::var("ROS_DISTRO").unwrap();

    Command::new("/bin/bash")
        .arg(format!("/opt/ros/{}/setup.bash", ros_distro))
        .status()
        .unwrap();

    Ok(())
}
