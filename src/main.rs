use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, Orientation, Entry, Button, Label};
use gtk4_layer_shell::{Edge,LayerShell, Layer};
use greetd_ipc::{Request, Response, AuthMessageType, codec::SyncCodec};
use std::process::exit;
use std::{
    env,
    os::unix::net::UnixStream,
    io::{BufReader, BufWriter},
    cell::RefCell,
    rc::Rc,
};

fn main() {
    let app = Application::builder()
        .application_id("ekah.scu.octobacillus")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    // Set up Layer Shell fullscreen window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("octobacillus")
        .default_width(1940)
        .default_height(1080)
        .build();
    
    window.init_layer_shell();
    window.set_layer(Layer::Bottom);
    window.auto_exclusive_zone_enable();
    window.fullscreen();
    window.set_decorated(false);
    window.set_namespace(Some("cynator"));

    for (edge, anchor) in [
        (Edge::Left, true),
        (Edge::Right, true),
        (Edge::Top, true),
        (Edge::Bottom, true),
    ] {
        window.set_anchor(edge, anchor);
    }

    // UI layout
    let vbox = GtkBox::new(Orientation::Vertical, 10);
    let status = Label::new(None);
    let username_entry = Entry::builder().placeholder_text("Username").build();
    let password_entry = Entry::builder().placeholder_text("Password").visibility(false).build();
    let login_button = Button::with_label("Login");

    vbox.append(&username_entry);
    vbox.append(&password_entry);
    vbox.append(&login_button);
    vbox.append(&status);
    window.set_child(Some(&vbox));
    window.show();

    // Login logic
    let status = Rc::new(status);
    let cmd = Rc::new(RefCell::new(None::<String>));
    let username_entry_clone = username_entry.clone();
    let password_entry_clone = password_entry.clone();
    let status_clone = status.clone();
    let cmd_clone = cmd.clone();

    login_button.connect_clicked(move |_| {
        let username = username_entry_clone.text().to_string();
        let password = password_entry_clone.text().to_string();
        let mut stream = match UnixStream::connect(env::var("GREETD_SOCK").unwrap()) {
            Ok(s) => s,
            Err(e) => {
                status_clone.set_text(&format!("Connection error: {e}"));
                return;
            }
        };

        let mut next_request = Request::CreateSession { username: username.clone() };
        let mut starting = false;

        loop {
            if let Err(e) = next_request.write_to(&mut stream) {
                status_clone.set_text(&format!("Write error: {e}"));
                break;
            }

            match Response::read_from(&mut stream) {
                Ok(Response::AuthMessage { auth_message, auth_message_type }) => {
                    let response = match auth_message_type {
                        AuthMessageType::Visible => Some(username.clone()),
                        AuthMessageType::Secret => Some(password.clone()),
                        _ => None,
                    };
                    next_request = Request::PostAuthMessageResponse { response };
                }
                Ok(Response::Success) => {
                    if starting {
                        // Success: exit app
                        gtk4::glib::idle_add_local_once(|| {
                            exit(0); // optional: close login window
                        });
                        break;
                    } else {
                        starting = true;
                        next_request = Request::StartSession {
                            env: vec![],
                            cmd: vec!["Hyprland".to_string()],
                        };
                    }
                }
                Ok(Response::Error { description, .. }) => {
                    status_clone.set_text(&format!("Auth failed: {description}"));
                    break;
                }
                Err(e) => {
                    status_clone.set_text(&format!("Response error: {e}"));
                    break;
                }
            }
        }
    });
}
