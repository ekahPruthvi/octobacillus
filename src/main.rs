use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, Orientation, Entry, Button, Label};
use gtk4_layer_shell::{Edge, LayerShell, Layer};
use greetd_ipc::{Request, Response, AuthMessageType, codec::SyncCodec};
use std::process::exit;
use std::{
    env,
    fs,
    os::unix::net::UnixStream,
    rc::Rc,
    time::Instant,
};
use chrono;

fn read_username_from_file() -> Option<String> {
    let path = "/usr/share/octobacillus/user.octo";
    let content = fs::read_to_string(path).ok()?;
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("name") {
            if let Some(eq_value) = rest.split('=').nth(1) {
                return Some(eq_value.trim().to_string());
            }
        }
    }
    None
}

fn slide_out_and_quit(window: &ApplicationWindow) {
    let win_clone = window.clone();
    let start_time = Instant::now();

    window.add_tick_callback(move |_, _| {
        let elapsed = start_time.elapsed().as_millis();
        let slide_y = (elapsed as f64 / 500.0).min(1.0); // 500ms animation
        let translate_y = 1080.0 * slide_y;

        win_clone.set_margin_top(translate_y as i32);

        if slide_y >= 1.0 {
            exit(0);
        }
        gtk4::glib::ControlFlow::Continue
    });
}

fn main() {
    let app = Application::builder()
        .application_id("ekah.scu.octobacillus")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
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
    window.set_namespace(Some("octobacillus_l"));

    for (edge, anchor) in [
        (Edge::Left, true),
        (Edge::Right, true),
        (Edge::Top, true),
        (Edge::Bottom, true),
    ] {
        window.set_anchor(edge, anchor);
    }

    let vbox = GtkBox::new(Orientation::Vertical, 10);
    let status = Label::new(None);
    let username_entry = Label::new(None);
    let password_entry = Entry::builder().placeholder_text("Password").visibility(false).build();
    let login_button = Button::with_label("Login");

    vbox.append(&username_entry);
    vbox.append(&password_entry);
    vbox.append(&login_button);
    vbox.append(&status);
    window.set_child(Some(&vbox));
    window.show();

    // Autofill last user if present
    let last_user = read_username_from_file();
    if let Some(u) = &last_user {
        username_entry.set_text(u);
        username_entry.set_visible(false);
        password_entry.grab_focus();
    }

    let status = Rc::new(status);
    let username_entry_rc = Rc::new(username_entry);
    let password_entry_rc = Rc::new(password_entry);
    let window_rc = Rc::new(window);

    login_button.connect_clicked({
        let username_entry = username_entry_rc.clone();
        let password_entry = password_entry_rc.clone();
        let status = status.clone();
        let window = window_rc.clone();
        move |_| {
            let username = if let Some(user) = read_username_from_file() {
                user
            } else {
                username_entry.text().to_string()
            };
            let password = password_entry.text().to_string();

            let mut stream = match UnixStream::connect(env::var("GREETD_SOCK").unwrap()) {
                Ok(s) => s,
                Err(e) => {
                    status.set_text(&format!("Connection error: {e}"));
                    return;
                }
            };

            let mut next_request = Request::CreateSession { username: username.clone() };
            let mut starting = false;

            loop {
                if let Err(e) = next_request.write_to(&mut stream) {
                    status.set_text(&format!("Write error: {e}"));
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
                            slide_out_and_quit(&window);
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
                        status.set_text(&format!("Login failed: {description}"));
                        let _ = Request::CancelSession.write_to(&mut stream);
                        break;
                    }
                    Err(e) => {
                        status.set_text(&format!("Response error: {e}"));
                        break;
                    }
                }
            }
        }
    });
}
