use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, Orientation, Entry, prelude::EntryExt, Label, CssProvider, glib, EventControllerKey, EventControllerMotion, Picture, Overlay, Button};
use gtk4_layer_shell::{Edge, LayerShell, Layer};
use greetd_ipc::{Request, Response, AuthMessageType, codec::SyncCodec};
use std::{
    env,
    fs,
    os::unix::net::UnixStream,
    rc::Rc,
    time::Instant,
};
use chrono::{Datelike, Local};
use std::cell::RefCell;
use std::f64::consts::PI;

fn make_label_bouncy(label: &Label, amplitude: f64, speed: f64) {
    let label_clone = label.clone();
    let start_time = Instant::now();

    label.add_tick_callback(move |_, _| {
        let elapsed = start_time.elapsed().as_secs_f64();
        let offset = (elapsed * speed * 2.0 * PI).sin() * amplitude;

        label_clone.set_margin_top(offset.max(0.0) as i32); // Prevent negative margin
        label_clone.set_margin_bottom((-offset).max(0.0) as i32);

        glib::ControlFlow::Continue
    });
}

fn typing_effect(label: &Label, text: &str, delay_ms: u64) {
    let label = label.clone();
    let chars: Vec<char> = text.chars().collect();
    let index = Rc::new(RefCell::new(0));

    let chars_rc = Rc::new(chars);

    glib::timeout_add_local(std::time::Duration::from_millis(delay_ms), move || {
        let i = *index.borrow();

        if i < chars_rc.len() {
            let current_text = chars_rc.iter().take(i + 1).collect::<String>();
            label.set_text(&current_text);
            *index.borrow_mut() += 1;
            glib::ControlFlow::Continue
        } else {
            glib::ControlFlow::Break
        }
    });
}

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

fn fade_out_and_quit(window: &ApplicationWindow) {
    let win_clone = window.clone();
    let start_time = Instant::now();

    window.add_tick_callback(move |_, _| {
        let elapsed = start_time.elapsed().as_millis();
        let t = (elapsed as f64 / 500.0).min(1.0); // duration: 500ms
        let eased = ease_in_out_cubic(1.0 - t); // fade from 1.0 to 0.0

        win_clone.set_opacity(eased);

        if t >= 1.0 {
            std::process::exit(0);
        }

        gtk4::glib::ControlFlow::Continue
    });
}

fn ease_in_out_cubic(t: f64) -> f64 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
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

    let css = CssProvider::new();
    css.load_from_data(
        "
        #time {
            font-family: Cantarell;
            font-size: 86px;
            letter-spacing: -2px;
            font-weight: 900;
            color: rgba(255, 255, 255, 0.5);
        }

        #user {
            font-family: Cantarell;
            font-size: 15px;
            font-weight: 900;
            color: rgba(0, 0, 0, 0.67);
        }

        #boxxy {
            background-color: rgba(0, 0, 0, 0.53);
            background: linear-gradient(
            -45deg,
            rgba(0, 240, 248, 0.17),
            rgba(248, 0, 182, 0.17),
            rgba(237, 245, 3, 0.17),
            rgba(2, 246, 193, 0.17),
            rgba(0, 240, 248, 0.17),
            rgba(149, 0, 248, 0.17)
            );
            background-size: 400% 400%;
            animation: gradient 30s ease infinite;
        }


        @keyframes gradient {
        0% {
            background-position: 0% 50%;
        }
        25% {
            background-position: 50% 100%;
        }
        50% {
            background-position: 100% 50%;
        }
        75% {
            background-position: 50% 0%;
        }
        100% {
            background-position: 0% 50%;
        }
        }

        #password {
            all: unset;
            padding: 10px;
            background-color: rgb(37, 37, 37);
            border-radius: 50px;
            border: 1px solid rgba(151, 151, 151, 0.53);
            color: white;
            caret-color: white;
        }

        .calendar-container {
            background-color:rgba(255, 255, 255, 0.32);
            border-radius: 50px;
            padding: 12px;
            border: 1px solid rgba(255, 255, 255, 0.18);
        }
        .day-label {
            all: unset;
            background-color: transparent;
            color: white;
            border: none;
            font-weight: 500;
            border-radius: 12px;
            padding: 2px;
            padding-right: 10px;
            padding-left: 10px;
            margin-right: 15px;
            margin-left: 15px;
        }
        .date-button {
            all: unset;
            background-color: transparent;
            color: white;
            border: none;
            font-weight: 500;
            border-radius: 12px;
            padding: 2px;
            padding-right: 10px;
            padding-left: 10px;
            margin-right: 20px;
            margin-left: 20px;
        }
        .date-button.today {
            background-color: rgba(255, 255, 255, 0.2);
            color: black;
            font-weight: bold;
        }

        label {
            transition: margin 0.1s ease-in-out;
        }


        ",  
    );
    
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().unwrap(),
        &css,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let overlay = Overlay::new();
    let boxxy = GtkBox::new(Orientation::Vertical, 10);
    overlay.add_overlay(&boxxy);
    boxxy.set_vexpand(true);
    boxxy.set_hexpand(true);
    boxxy.set_valign(gtk4::Align::Fill);
    boxxy.set_widget_name("boxxy");

    let gif_path = "/usr/share/octobacillus/bg.png";
    let file = gtk4::gio::File::for_path(gif_path);

    let gif = Picture::for_file(&file);
    gif.set_widget_name("gif-bg");
    gif.set_hexpand(true);
    gif.set_vexpand(true);
    gif.set_halign(gtk4::Align::Fill);
    gif.set_valign(gtk4::Align::Fill);

    overlay.set_child(Some(&gif));
    
    let status = Label::new(None);
    let username_entry = Label::new(None);
    username_entry.set_widget_name("user");
    let password_entry = Entry::builder().placeholder_text("Enter Password").visibility(false).build();
    password_entry.set_widget_name("password");
    gtk4::prelude::EntryExt::set_alignment(&password_entry, 0.5);
    password_entry.set_hexpand(true);
    password_entry.set_vexpand(true);
    password_entry.set_halign(gtk4::Align::Center);

    let time = Label::new(Some("cynageOS"));
    time.set_widget_name("time");
    time.set_margin_top(100);
    let label_weak = time.downgrade();
    let mut prev = String::from("cynageOS");

    glib::timeout_add_seconds_local(1, move || {
        if let Some(label) = label_weak.upgrade() {
            let now = Local::now();
            let current = now.format("%I:%M %p").to_string();

            if prev != current {
                prev = current.clone();
                typing_effect(&label, &current, 50);
            }

            glib::ControlFlow::Continue
        } else {
            glib::ControlFlow::Break
        }
    });
    boxxy.append(&time);

    let container = GtkBox::new(Orientation::Vertical, 8);
    container.add_css_class("calendar-container");

    let weekdays = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];
    let days_row = GtkBox::new(Orientation::Horizontal, 10);
    for day in weekdays {
        let label = Label::new(Some(day));
        label.add_css_class("day-label");
        days_row.append(&label);
    }

    let today = Local::now().date_naive();
    let start_of_week = today - chrono::Duration::days((today.weekday().num_days_from_monday()) as i64);

    let dates_row = GtkBox::new(Orientation::Horizontal, 10);
    for i in 0..7 {
        let date = start_of_week + chrono::Duration::days(i);
        let day_label = Button::with_label(&format!("{}", date.day()));
        day_label.add_css_class("date-button");

        if date == today {
            day_label.add_css_class("today");
        }

        dates_row.append(&day_label);
    }

    container.append(&days_row);
    container.append(&dates_row);
    container.set_hexpand(true);
    container.set_margin_top(10);
    container.set_halign(gtk4::Align::Center);

    boxxy.append(&container);

    window.set_child(Some(&overlay));
    window.show();

    let workingbox = GtkBox::new(Orientation::Vertical, 10);
    workingbox.set_vexpand(true);
    workingbox.set_valign(gtk4::Align::End);
    workingbox.set_widget_name("workin");
    workingbox.append(&username_entry);
    boxxy.append(&workingbox);

    let pass_box = GtkBox::new(Orientation::Vertical, 0);
    pass_box.set_height_request(5);
    pass_box.set_widget_name("hover-mee");
    workingbox.append(&pass_box);
    workingbox.append(&status);

    let pass_box_weak_key = pass_box.downgrade();
    let passwork_entry_outer_key = password_entry.clone();

    let key_controller = EventControllerKey::new();
    key_controller.connect_key_pressed(move |_, _, _, _| {
        let pass_box_opt = pass_box_weak_key.upgrade();
        let pass_box_weak_inner = pass_box_opt.clone();
        let password_entry_clone = passwork_entry_outer_key.clone();

        glib::timeout_add_local(std::time::Duration::from_millis(10), move || {
            if let Some(pass_box) = &pass_box_weak_inner {
                let current = pass_box.height_request();
                if current < 30 {
                    pass_box.set_height_request(current + 1);
                    pass_box.set_margin_bottom(current + 2);
                    glib::ControlFlow::Continue
                } else {
                    pass_box.append(&password_entry_clone);
                    password_entry_clone.grab_focus();
                    glib::ControlFlow::Break
                }
            } else {
                glib::ControlFlow::Break
            }
        });
        
        gtk4::glib::Propagation::Stop
    });

    window.add_controller(key_controller);
    
    let pass_box_weak = pass_box.downgrade();
    // Add motion controller to workingbox
    let motion_controller = EventControllerMotion::new();
    workingbox.add_controller(motion_controller.clone());
    let passwork_entry_outer = password_entry.clone();

    motion_controller.connect_enter(move |_, _, _| {
        let pass_box_weak_inner = pass_box_weak.clone();
        let password_entry_clone = passwork_entry_outer.clone();

        glib::timeout_add_local(std::time::Duration::from_millis(10), move || {
            if let Some(pass_box) = pass_box_weak_inner.upgrade() {
                let current = pass_box.height_request();
                if current < 30 {
                    pass_box.set_height_request(current + 1);
                    pass_box.set_margin_bottom(current + 2);
                    glib::ControlFlow::Continue
                } else {
                    pass_box.append(&password_entry_clone);
                    password_entry_clone.grab_focus();
                    glib::ControlFlow::Break
                }
            } else {
                glib::ControlFlow::Break
            }
        });
    });

    let last_user = read_username_from_file();
    make_label_bouncy(&username_entry, 10.0, 0.7);
    if let Some(u) = &last_user {
        username_entry.set_text(&format!("welcome, {}", u));
        username_entry.set_visible(true);
    }

    let status = Rc::new(status);
    let username_entry_rc = Rc::new(username_entry);
    let password_entry_rc = Rc::new(password_entry.clone());
    let window_rc = Rc::new(window);

    password_entry.connect_activate(move |_entry| {
        let username_entry = username_entry_rc.clone();
        let password_entry = password_entry_rc.clone();
        let status = status.clone();
        let window = window_rc.clone();

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
                Ok(Response::AuthMessage { auth_message: _, auth_message_type }) => {
                    let response = match auth_message_type {
                        AuthMessageType::Visible => Some(username.clone()),
                        AuthMessageType::Secret => Some(password.clone()),
                        _ => None,
                    };
                    next_request = Request::PostAuthMessageResponse { response };
                }
                Ok(Response::Success) => {
                    if starting {

                        fade_out_and_quit(&window);
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
    });

}
