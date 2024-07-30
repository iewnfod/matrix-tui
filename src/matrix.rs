use std::sync::Mutex;

use matrix_sdk::{config::SyncSettings, ruma::events::{room::message::SyncRoomMessageEvent, MessageLikeEventType}, Client, ServerName};
use lazy_static::lazy_static;

use crate::save::{BUNDLE_ID, SAVING};

lazy_static! {
    pub static ref MATRIX_CLIENT: Mutex<MatrixClient> = Mutex::new(MatrixClient::new());
}

pub async fn login(server: String, username: String, password: String) -> Option<Client> {
	set_error_message("");
	set_loading(true);

	if server.is_empty() || username.is_empty() || password.is_empty() {
		set_error_message("Missing blank");
		set_loading(false);
		return None;
	}

	let server_name = match ServerName::parse(&server) {
		Ok(server_name) => server_name,
		Err(_) => {
			set_error_message("Invalid server name");
			set_loading(false);
			return None;
		}
	};

	set_info_message("Connecting to server");
	let client = match Client::builder().server_name(&server_name).build().await {
		Ok(client) => client,
		Err(e) => {
			set_error_message(format!("Failed to connect to server: {}", e.to_string()));
			set_loading(false);
			return None;
		}
	};

	let device_id = client.device_id()?;
	set_info_message(format!("Registering device: {:?}", device_id));
	let _ = client.rename_device(
		device_id,
		"Matrix Tui"
	).await;

	set_info_message("Logging in");
	match client.matrix_auth()
		.login_username(username.clone(), &password)
		.send().await {
			Ok(_) => {},
			Err(e) => {
				set_error_message(format!("Failed to login: {}", e.to_string()));
				set_loading(false);
				return None;
			}
		};

	if let Some(token) = client.access_token() {
		let mut saving = SAVING.lock().unwrap();
		saving.token = token;
		saving.username = username.clone();
		saving.server = server.clone();
		saving.save();
	}

	connect(&client).await;

	set_loading(false);

	Some(client)
}

pub async fn login_with_token(server: &str, token: &str) -> Option<Client> {
	set_loading(true);

	let server_name = match ServerName::parse(server) {
		Ok(server_name) => server_name,
		Err(_) => {
			set_error_message("Invalid server name");
			set_loading(false);
			return None;
		}
	};

	set_info_message("Connecting to server");
	let client = match Client::builder().server_name(&server_name).build().await {
		Ok(client) => client,
		Err(e) => {
			set_error_message(format!("Failed to connect to server: {}", e.to_string()));
			set_loading(false);
			return None;
		}
	};

	set_info_message("Logging in with token");
	match client.matrix_auth().login_token(token).send().await {
		Ok(_) => {},
		Err(e) => {
			set_error_message(format!("Failed to login with token: {}", e.to_string()));
			set_loading(false);
			return None;
		}
	};

	connect(&client).await;

	set_loading(false);

	Some(client)
}

pub fn set_error_message<T: ToString>(msg: T) {
    let mut client = MATRIX_CLIENT.lock().unwrap();
    client.error_message = msg.to_string();
}

fn set_loading(loading: bool) {
    let mut client = MATRIX_CLIENT.lock().unwrap();
    client.loading = loading;
}

fn set_connected(connected: bool) {
	let mut client = MATRIX_CLIENT.lock().unwrap();
	client.connected = connected;
}

fn set_info_message<T: ToString>(msg: T) {
	let mut client = MATRIX_CLIENT.lock().unwrap();
    client.info_message = msg.to_string();
}

pub fn get_matrix_client() -> MatrixClient {
    let client = MATRIX_CLIENT.lock().unwrap();
    client.clone()
}

#[derive(Debug, Clone)]
pub struct MatrixClient {
	pub error_message: String,
	pub info_message: String,
	pub connected: bool,
	pub loading: bool
}

impl MatrixClient {
	fn new() -> Self {
		Self {
			error_message: String::new(),
			info_message: String::new(),
			connected: false,
			loading: false,
		}
	}
}

async fn connect(client: &Client) {
	client.add_event_handler(|ev: SyncRoomMessageEvent| async move {
		match ev.event_type() {
			MessageLikeEventType::Message => {

			}
			_ => {}
		}
	});

	set_info_message("Syncing with server");
	match client.sync(SyncSettings::default()).await {
		Ok(_) => {},
		Err(e) => {
			set_error_message(format!("Failed to sync with server: {}", e.to_string()));
			return;
		}
	};

	set_connected(true);
}
