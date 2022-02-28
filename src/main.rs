use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use teleop::{
    teleop_server::{Teleop, TeleopServer},
    CommandAck, CommandRequest,
};
use tonic::{transport::Server, Request, Response, Status};

use r2r::{geometry_msgs, Node, QosProfile};

pub mod teleop {
    tonic::include_proto!("teleop");
}

pub struct TeleopService {
    node: Arc<Mutex<Node>>,
}

fn match_request(request: CommandRequest) -> geometry_msgs::msg::Twist {
    let mut linear = geometry_msgs::msg::Vector3 {
        x: 0 as f64,
        y: 0 as f64,
        z: 0 as f64,
    };

    let mut angular = geometry_msgs::msg::Vector3 {
        x: 0 as f64,
        y: 0 as f64,
        z: 0 as f64,
    };

    match request.linear {
        Some(data) => {
            linear = geometry_msgs::msg::Vector3 {
                x: data.x as f64,
                y: data.y as f64,
                z: data.z as f64,
            };
        }
        None => {
            linear = geometry_msgs::msg::Vector3 {
                x: 0 as f64,
                y: 0 as f64,
                z: 0 as f64,
            };
        }
    }

    match request.angular {
        Some(data) => {
            angular = geometry_msgs::msg::Vector3 {
                x: data.x as f64,
                y: data.y as f64,
                z: data.z as f64,
            };
        }
        None => {
            angular = geometry_msgs::msg::Vector3 {
                x: 0 as f64,
                y: 0 as f64,
                z: 0 as f64,
            };
        }
    }

    geometry_msgs::msg::Twist { linear, angular }
}

#[tonic::async_trait]
impl Teleop for TeleopService {
    async fn send_command(
        &self,
        request: Request<CommandRequest>,
    ) -> Result<Response<CommandAck>, Status> {
        let mut node = self.node.lock().unwrap();

        let publisher = node
            .create_publisher::<geometry_msgs::msg::Twist>("/cmd_vel", QosProfile::default())
            .unwrap();

        let msg = match_request(request.into_inner());

        println!("got {:?}", msg);

        publisher.publish(&msg).unwrap();

        let message = teleop::CommandAck { success: true };

        Ok(Response::new(message))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = r2r::Context::create()?;
    let node = r2r::Node::create(ctx, "teleop_node", "")?;
    let arc_node = Arc::new(Mutex::new(node));

    let addr = "127.0.0.1:50051".parse()?;

    let teleop_service = TeleopService {
        node: arc_node.clone(),
    };

    tokio::task::spawn_blocking(move || loop {
        arc_node
            .lock()
            .unwrap()
            .spin_once(Duration::from_millis(100))
    });

    Server::builder()
        .add_service(TeleopServer::new(teleop_service))
        .serve(addr)
        .await
        .unwrap();

    Ok(())
}
