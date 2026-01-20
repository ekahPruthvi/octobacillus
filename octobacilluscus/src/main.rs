use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, Orientation, LevelBar, Label, CssProvider, glib, EventControllerMotion};
use gtk4_layer_shell::{Edge, LayerShell, Layer};
use std::{
    rc::Rc,
    time::Instant,
};
use chrono::Local;
use std::cell::RefCell;
use std::f64::consts::PI;
use std::process::exit;

// make hide cursor

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

fn main() {
    let app = Application::builder()
        .application_id("ekah.scu.octobacilluscus")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("octobacilluscus")
        .default_width(1940)
        .default_height(1080)
        .build();

    window.init_layer_shell();
    window.set_layer(Layer::Top);
    window.auto_exclusive_zone_enable();
    window.set_can_focus(true);
    window.grab_focus();
    window.fullscreen();
    window.set_decorated(false);
    window.set_namespace(Some("octobacilluscus_l"));
    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);

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
        window {
            background-color: rgba(0, 0, 0, 0);
        }

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
            color: rgba(255, 255, 255, 0.68);
        }

        #boxxy {
            background-color: rgba(0, 0, 0, 0.92);
        }

        label {
            transition: margin 0.1s ease-in-out;
        }

        .lock-toggle {
            all:unset;
            min-height: 2px;
            min-width: 300px;
            border-radius: 50px;
            background-color: rgba(0, 0, 0, 0);
            padding: 5px;
        }

        .levelbar trough {
            border-radius: 50px;
            background: linear-gradient(to right, rgb(0, 0, 0), #555);
        }

        .levelbar block.filled {
        background-color: white;
        border-radius: 50px;
        background-position: center;
        background-repeat: no-repeat;
        }
    ",  
    );
    
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().unwrap(),
        &css,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let boxxy = GtkBox::new(Orientation::Vertical, 10);
    boxxy.set_vexpand(true);
    boxxy.set_hexpand(true);
    boxxy.set_valign(gtk4::Align::Fill);
    boxxy.set_widget_name("boxxy");
    
    let status = Label::new(None);
    let username_entry = Label::new(None);
    username_entry.set_widget_name("user");
    let password_entry = LevelBar::new();
    password_entry.set_orientation(Orientation::Horizontal);
    password_entry.set_min_value(0.0);
    password_entry.set_max_value(1.0);
    password_entry.set_value(0.0);
    password_entry.set_inverted(false); // fill left-to-right
    password_entry.add_css_class("lock-toggle");
    password_entry.set_widget_name("password");
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

    window.set_child(Some(&boxxy));
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
    pass_box.append(&password_entry);
    workingbox.append(&pass_box);
    workingbox.append(&status);
    
    // Add motion controller to workingbox
    let motion_controller = EventControllerMotion::new();
    workingbox.add_controller(motion_controller.clone());

    motion_controller.connect_enter(move |_, _, _| {
        let password_entry_inner = password_entry.clone();

        glib::timeout_add_local(std::time::Duration::from_millis(10), move || {
            let current = password_entry_inner.value();
            if current < 1.0 {
                password_entry_inner.set_value((current + 0.01).min(1.0));
                glib::ControlFlow::Continue
            } else {
                exit(0);
            }
        });
    });

    make_label_bouncy(&username_entry, 10.0, 0.7);
    username_entry.set_markup("<b>hover to</b><i> unlock</i>");
    username_entry.set_visible(true);
}
