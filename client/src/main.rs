mod client;
mod packets;

use tracing::{error, info};
extern crate gtk;
extern crate serializer;
use crate::client::Client;
use gtk::prelude::*;
use gtk::{glib, ButtonsType, NONE_ADJUSTMENT};
use serializer::{new_connect_flag, new_payload_connect};
use std::cell::RefCell;
use std::collections::HashMap;
use std::net::{Shutdown, TcpStream};
use std::os::unix::process::CommandExt;
use std::process::Command;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

fn main() {
    // Creo un subscriber no bloqueante para todos los tipos de eventos que escriba en un archivo .log con rotacion diaria.
    let file_appender = tracing_appender::rolling::daily("logs", "client_log.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt().with_writer(non_blocking).init();
    info!("Inicio de cliente.");

    let application =
        gtk::Application::new(Some("com.github.gtk-rs.examples.basic"), Default::default());
    let config;
    match client::decode_config() {
        Ok(c) => {
            config = c;
        }
        Err(_) => {
            application.connect_activate(error_ui);
            application.run();
            return;
        }
    }
    let address = config[0][1].to_string() + ":" + config[1][1].to_string().as_str();
    match TcpStream::connect(address) {
        Ok(t) => {
            let _res = t.shutdown(Shutdown::Both);
        }
        Err(_) => {
            application.connect_activate(error_ui);
            application.run();
            return;
        }
    }
    application.connect_activate(build_ui);

    application.run();
}

fn error_ui(application: &gtk::Application) {
    let _windows: Rc<RefCell<HashMap<usize, glib::WeakRef<gtk::Window>>>> =
        Rc::new(RefCell::new(HashMap::new()));
    let window = gtk::ApplicationWindow::new(application);

    // Traigo el config para mostrarlo en la UI.

    window.set_title("Error al iniciar");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(200, 100);
    let error_label = gtk::Label::new(Some("Error al conectarse con servidor. \n Verifique su archivo de configuración o intente más tarde."));
    let ip_label = gtk::Label::new(Some("Server Ip"));
    let ip_entry = gtk::Entry::new();
    // ip_entry.set_text("127.0.0.1");
    //ip_entry.set_editable(false);
    // ip_entry.set_placeholder_text(Some("127.0.0.1"));
    let ip_layout = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    ip_layout.add(&ip_label);
    ip_layout.add(&ip_entry);

    let port_label = gtk::Label::new(Some("Port"));
    let port_entry = gtk::Entry::new();
    // port_entry.set_text("7878");
    //port_entry.set_editable(false);
    // port_entry.set_placeholder_text(Some("7878"));

    match client::decode_config() {
        Ok(c) => {
            ip_entry.set_text(c[0][1].to_string().as_str());
            port_entry.set_text(c[1][1].to_string().as_str());
        }
        Err(_) => {
            ip_entry.set_text("127.0.0.1");
            port_entry.set_text("7878");
        }
    }
    let port_layout = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    port_layout.add(&port_label);
    port_layout.add(&port_entry);

    let save_changes_buttons = gtk::Button::with_label("Save and Restart");
    let save_button_layout = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    save_button_layout.add(&save_changes_buttons);

    let save_button_dialog = gtk::MessageDialog::new(
        None::<&gtk::Window>,
        gtk::DialogFlags::DESTROY_WITH_PARENT,
        gtk::MessageType::Info,
        ButtonsType::Ok,
        "Client config updated!",
    );

    save_button_dialog.connect_button_press_event(|save_button_dialog, _| {
        save_button_dialog.hide();
        gtk::Inhibit(true)
    });
    save_changes_buttons.connect_clicked(glib::clone!(@weak save_button_dialog, @weak ip_entry
                                    ,@weak port_entry => move |_| {
            let ip = if ip_entry.text().is_empty() { "".to_string() } else { ip_entry.text().to_string() };
            let port = if port_entry.text().is_empty() { "".to_string() } else { port_entry.text().to_string() };
            client::update_changes(ip, port);
            Command::new("./target/debug/client").exec();
    }));
    let ok_button = gtk::Button::with_label("Ok");
    ok_button.connect_clicked(glib::clone!(@weak window => move |_| {
        window.close();
    }));
    error_label.set_justify(gtk::Justification::Center);
    let layout = gtk::Box::new(gtk::Orientation::Vertical, 5);
    layout.add(&error_label);
    layout.add(&ip_layout);
    layout.add(&port_layout);
    layout.add(&save_button_layout);
    layout.add(&ok_button);
    window.add(&layout);
    window.show_all();
}

fn build_ui(application: &gtk::Application) {
    // Creo el cliente aqui para poder usar sus thread.
    let client = client::Client::new();
    // Write necesito que sea de esa forma porque voy a tener que clonarlo cada vez que
    // quiera que un boton envie un mensaje.
    let socket = client.socket;
    let read = Arc::new(Mutex::new(
        socket.try_clone().expect("couldn't clone the socket"),
    ));
    let write = Arc::new(Mutex::new(
        socket.try_clone().expect("couldn't clone the socket"),
    ));

    let _windows: Rc<RefCell<HashMap<usize, glib::WeakRef<gtk::Window>>>> =
        Rc::new(RefCell::new(HashMap::new()));
    let window = gtk::ApplicationWindow::new(application);

    window.set_title("Talleres de Cordoba");
    window.set_border_width(10);
    window.set_position(gtk::WindowPosition::Center);
    window.set_default_size(400, 200);

    // CONNECT
    let con_header = gtk::HeaderBar::new();
    con_header.set_title(Some("Connection"));

    let ip_label = gtk::Label::new(Some("Server Ip"));
    let ip_entry = gtk::Entry::new();
    // ip_entry.set_text("127.0.0.1");
    //ip_entry.set_editable(false);
    // ip_entry.set_placeholder_text(Some("127.0.0.1"));
    let ip_layout = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    ip_layout.add(&ip_label);
    ip_layout.add(&ip_entry);

    let port_label = gtk::Label::new(Some("Port"));
    let port_entry = gtk::Entry::new();
    // port_entry.set_text("7878");
    //port_entry.set_editable(false);
    // port_entry.set_placeholder_text(Some("7878"));

    match client::decode_config() {
        Ok(c) => {
            ip_entry.set_text(c[0][1].to_string().as_str());
            port_entry.set_text(c[1][1].to_string().as_str());
        }
        Err(_) => {
            ip_entry.set_text("127.0.0.1");
            port_entry.set_text("7878");
        }
    }

    let port_layout = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    port_layout.add(&port_label);
    port_layout.add(&port_entry);

    let save_changes_buttons = gtk::Button::with_label("Save Port and IP");
    let save_button_layout = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    save_button_layout.add(&save_changes_buttons);

    let save_button_dialog = gtk::MessageDialog::new(
        None::<&gtk::Window>,
        gtk::DialogFlags::DESTROY_WITH_PARENT,
        gtk::MessageType::Info,
        ButtonsType::Ok,
        "Client config updated!",
    );

    let connected_status = gtk::Label::new(Some("Status"));
    let connected_entry = gtk::Entry::new();

    let connected_layout = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    connected_layout.add(&connected_status);
    connected_layout.add(&connected_entry);

    connected_entry.set_text("Disconnected");
    connected_entry.set_editable(false);

    save_button_dialog.connect_button_press_event(|save_button_dialog, _| {
        save_button_dialog.hide();
        gtk::Inhibit(true)
    });
    save_changes_buttons.connect_clicked(glib::clone!(@weak save_button_dialog, @weak ip_entry
                                    ,@weak port_entry, @weak connected_entry => move |_| {
        if connected_entry.text() == *"Connected"{
            save_button_dialog.set_text(Some("No puedes cambiar la configuracion estando conectado!"));
            save_button_dialog.set_message_type(gtk::MessageType::Warning);
            save_button_dialog.show_all();
            return;
        }
        let ip = if ip_entry.text().is_empty() { "".to_string() } else { ip_entry.text().to_string() };
        let port = if port_entry.text().is_empty() { "".to_string() } else { port_entry.text().to_string() };
        client::update_changes(ip, port);
        Command::new("./target/debug/client").exec();
    }));
    let client_id_label = gtk::Label::new(Some("Client ID"));
    let client_id_entry = gtk::Entry::new();
    client_id_entry.set_placeholder_text(Some("123"));
    let client_id_layout = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    client_id_layout.add(&client_id_label);
    client_id_layout.add(&client_id_entry);

    let clean_session_label = gtk::Label::new(Some("Clean Session"));
    let clean_session_cb = gtk::CheckButton::new();
    let clean_s_layout = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    clean_s_layout.add(&clean_session_label);
    clean_s_layout.add(&clean_session_cb);

    let will_flag_label = gtk::Label::new(Some("Will Flag"));
    let will_flag_cb = gtk::CheckButton::new();
    let will_flag_layout = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    will_flag_layout.add(&will_flag_label);
    will_flag_layout.add(&will_flag_cb);

    let willqos_label = gtk::Label::new(Some("Will"));
    let willqos0_label = gtk::Label::new(Some("QoS 0"));
    let willqos1_label = gtk::Label::new(Some("QoS 1"));
    let willqos_switch = gtk::Switch::new();
    willqos_switch.set_hexpand(false);
    willqos_switch.set_size_request(20, 30);
    let willqos_layout = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    willqos_layout.add(&willqos_label);
    willqos_layout.add(&willqos0_label);
    willqos_layout.add(&willqos_switch);
    willqos_layout.add(&willqos1_label);

    let will_retain_label = gtk::Label::new(Some("Will Retain"));
    let will_retain_cb = gtk::CheckButton::new();
    let will_ret_layout = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    will_ret_layout.add(&will_retain_label);
    will_ret_layout.add(&will_retain_cb);

    let username_flag_label = gtk::Label::new(Some("Username Flag"));
    let username_flag_cb = gtk::CheckButton::new();
    let username_flag_layout = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    username_flag_layout.add(&username_flag_label);
    username_flag_layout.add(&username_flag_cb);

    let password_flag_label = gtk::Label::new(Some("Password Flag"));
    let password_flag_cb = gtk::CheckButton::new();
    let password_flag_layout = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    password_flag_layout.add(&password_flag_label);
    password_flag_layout.add(&password_flag_cb);

    let keep_alive_label = gtk::Label::new(Some("Keep Alive (s)"));
    let keep_alive_spin = gtk::SpinButton::with_range(0.00, 65535.00, 1.0);
    let keep_alive_layout = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    keep_alive_layout.add(&keep_alive_label);
    keep_alive_layout.add(&keep_alive_spin);

    let connect_label = gtk::Label::new(Some("Connect Packet"));
    let connect_username_entry = gtk::Entry::new();
    connect_username_entry.set_placeholder_text(Some("Username"));
    let connect_password_entry = gtk::Entry::new();
    connect_password_entry.set_placeholder_text(Some("Password"));
    let connect_lwm_entry = gtk::Entry::new();
    connect_lwm_entry.set_placeholder_text(Some("Last Will Message"));
    let connect_lwt_entry = gtk::Entry::new();
    connect_lwt_entry.set_placeholder_text(Some("Last Will Topic"));

    let pack_layout = gtk::Box::new(gtk::Orientation::Vertical, 5);
    pack_layout.add(&connect_username_entry);
    pack_layout.add(&connect_password_entry);
    pack_layout.add(&connect_lwm_entry);
    pack_layout.add(&connect_lwt_entry);

    let con_pack_layout = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    con_pack_layout.add(&connect_label);
    con_pack_layout.add(&pack_layout);

    let button_connect = gtk::Button::with_label("Connect");
    let connect_dialog = gtk::MessageDialog::new(
        None::<&gtk::Window>,
        gtk::DialogFlags::DESTROY_WITH_PARENT,
        gtk::MessageType::Info,
        ButtonsType::Ok,
        "Connected unsuccesfully!",
    );

    connect_dialog.connect_button_press_event(|connect_dialog, _| {
        connect_dialog.hide();
        gtk::Inhibit(true)
    });

    //Debemos clonar el mutex por cada vez que querramos hacer algun envio de mensaje.
    let write_connect = write.clone();
    let _read_connect = read.clone();



    button_connect.connect_clicked(glib::clone!(@weak connect_dialog, @weak clean_session_cb,
                                                   @weak will_flag_cb, @weak will_retain_cb,
                                                   @weak username_flag_cb, @weak password_flag_cb,
                                                   @weak willqos_switch, @weak ip_entry,
                                                   @weak client_id_entry, @weak connect_lwt_entry,
                                                   @weak connect_lwm_entry, @weak connect_username_entry,
                                                   @weak connect_password_entry, @weak connected_entry,
                                                   @weak keep_alive_spin => move |_| {
        if client_id_entry.text().is_empty() {
            connect_dialog.set_text(Some("El campo Client ID es obligatorio."));
            connect_dialog.set_message_type(gtk::MessageType::Warning);
            connect_dialog.show_all();
            return;
        }
        if username_flag_cb.is_active() && connect_username_entry.text().is_empty() {
            connect_dialog.set_text(Some("El campo Username es obligatorio si activa su flag."));
            connect_dialog.set_message_type(gtk::MessageType::Warning);
            connect_dialog.show_all();
            return;
        }
        if ! username_flag_cb.is_active() && ! connect_username_entry.text().is_empty() {
            connect_dialog.set_text(Some("No debe proporcionar Username si no activa su flag."));
            connect_dialog.set_message_type(gtk::MessageType::Warning);
            connect_dialog.show_all();
            return;
        }
        if password_flag_cb.is_active() && !username_flag_cb.is_active() {
            connect_dialog.set_text(Some("Si el flag Password está activo, el de Username también debe estarlo."));
            connect_dialog.set_message_type(gtk::MessageType::Warning);
            connect_dialog.show_all();
            return;
        }
        if password_flag_cb.is_active() && connect_password_entry.text().is_empty() {
            connect_dialog.set_text(Some("El campo Password es obligatorio si activa su flag."));
            connect_dialog.set_message_type(gtk::MessageType::Warning);
            connect_dialog.show_all();
            return;
        }
        if ! password_flag_cb.is_active() && ! connect_password_entry.text().is_empty() {
            connect_dialog.set_text(Some("No debe proporcionar Password si no activa su flag."));
            connect_dialog.set_message_type(gtk::MessageType::Warning);
            connect_dialog.show_all();
            return;
        }
        if will_flag_cb.is_active() && (connect_lwt_entry.text().is_empty() || connect_lwm_entry.text().is_empty()) {
            connect_dialog.set_text(Some("Si el Will Flag está activo, debe proporcionar Last Will Topic y Message."));
            connect_dialog.set_message_type(gtk::MessageType::Warning);
            connect_dialog.show_all();
            return;
        }
        if ! will_flag_cb.is_active() && (! connect_lwt_entry.text().is_empty() || ! connect_lwm_entry.text().is_empty()) {
            connect_dialog.set_text(Some("Si el Will Flag está inactivo, no debe proporcionar Last Will Topic y Message."));
            connect_dialog.set_message_type(gtk::MessageType::Warning);
            connect_dialog.show_all();
            return;
        }
        if ! will_flag_cb.is_active() && will_retain_cb.is_active() {
            connect_dialog.set_text(Some("Si el Will Flag está inactivo, no debe activar Will Retain."));
            connect_dialog.set_message_type(gtk::MessageType::Warning);
            connect_dialog.show_all();
            return;
        }
        // Como no implementamos QoS2 el primer bit va a ser siempre 0.
        let willqos_b1 = false;
        let mut willqos_b2 = false;
        if willqos_switch.is_active()
        {
            willqos_b2 = true;
        }
        let connect_flag = new_connect_flag(
            Some(clean_session_cb.is_active()),
            Some(will_flag_cb.is_active()),
            Some(willqos_b1),
            Some(willqos_b2),
            Some(will_retain_cb.is_active()),
            Some(password_flag_cb.is_active()),
            Some(username_flag_cb.is_active()),
        )
        .ok()
        .unwrap();
        let client_id = if client_id_entry.text().is_empty() { "".to_string() } else { client_id_entry.text().to_string() };
        let connect_lwt = if connect_lwt_entry.text().is_empty() { "".to_string() } else { connect_lwt_entry.text().to_string() };
        let connect_lwm = if connect_lwm_entry.text().is_empty() { "".to_string() } else { connect_lwm_entry.text().to_string() };
        let connect_username = if connect_username_entry.text().is_empty() { "".to_string() } else { connect_username_entry.text().to_string() };
        let connect_password = if connect_password_entry.text().is_empty() { "".to_string() } else { connect_password_entry.text().to_string() };
        let keep_alive = if keep_alive_spin.text().is_empty() { 0 } else {
                match u16::from_str(keep_alive_spin.text().as_str()) {
                    Ok(res) => res,
                    Err(_) => {
                        connect_dialog.set_text(Some("Invalid Keep Alive!"));
                        connect_dialog.set_message_type(gtk::MessageType::Warning);
                        connect_dialog.show_all();
                        return;
                    }
                }
            };

        let connect_payload = new_payload_connect(
            client_id,
            connect_lwt,
            connect_lwm,
            connect_username,
            connect_password,
            keep_alive
        )
            .ok()
            .unwrap();
        if connected_entry.text() == *"Connected" {
            connect_dialog.set_text(Some("Ya estas conectado!"));
            connect_dialog.set_message_type(gtk::MessageType::Warning);
            connect_dialog.show_all()
        } else {
            let mut connect;
            let write_ping = write_connect.clone();
            match write_connect.lock() {
                Ok(r) => {connect = r}
                Err(e) => {panic!("{:?}",e)}
            }
            match packets::send_connect(&mut connect, connect_flag, connect_payload) {
                    Ok(_) => {
                        match u64::from_str(keep_alive_spin.text().as_str()) {
                            Ok(keep_alive_secs) => {
                                if keep_alive_secs > 0 {
                                    glib::timeout_add(Duration::from_secs(keep_alive_secs), move || {
                                        let mut ping;
                                        match write_ping.lock() {
                                            Ok(r) => {ping = r}
                                            Err(e) => {panic!("{:?}",e)}
                                        };
                                        if packets::send_pingreq(&mut ping).is_ok() {}
                                        glib::Continue(true)
                                    });
                                    info!("Se enviara un PINGREQ cada {:?} segundos.", keep_alive_secs);
                                }

                            },
                            Err(_) => {
                                error!("Error al convertir keep alive secs.");
                            }
                        };

                    }, // Se va a mostrar la respuesta cuando se reciba.
                    Err(_) => {
                        connected_entry.set_text("Disconnected");
                        connect_dialog.set_text(Some("Error al enviar paquete."));
                        connect_dialog.set_message_type(gtk::MessageType::Error);
                        connect_dialog.show_all()
                    }
            }
        }
    }));

    let button_disconnect = gtk::Button::with_label("Disconnect");
    let disconnect_dialog = gtk::MessageDialog::new(
        None::<&gtk::Window>,
        gtk::DialogFlags::DESTROY_WITH_PARENT,
        gtk::MessageType::Info,
        ButtonsType::Ok,
        "Disconnected unsuccesfully!",
    );
    disconnect_dialog.connect_button_press_event(|disconnect_dialog, _| {
        disconnect_dialog.hide();
        Command::new("./target/debug/client").exec();
        gtk::Inhibit(true)
    });
    let write_disconnect = write.clone();
    let _read_disconnect = read.clone();
    button_disconnect.connect_clicked(glib::clone!(@weak disconnect_dialog, @weak clean_session_cb,
                                                      @weak will_flag_cb, @weak will_retain_cb, @weak port_entry,
                                                      @weak username_flag_cb, @weak password_flag_cb,
                                                      @weak willqos_switch, @weak ip_entry,
                                                      @weak client_id_entry, @weak connect_lwt_entry,
                                                      @weak connect_lwm_entry, @weak connect_username_entry,
                                                      @weak connect_password_entry, @weak connected_entry => move |_| {
        if connected_entry.text() == *"Disconnected" {
            disconnect_dialog.set_text(Some("No estas conectado!"));
            disconnect_dialog.set_message_type(gtk::MessageType::Warning);
        } else {
            let resp = packets::send_disconnect(&mut write_disconnect.lock().unwrap());
            match resp {
                    Ok(_) => {
                        disconnect_dialog.set_text(Some("Disconnected successfully!"));
                        disconnect_dialog.set_message_type(gtk::MessageType::Info);
                        // Limpio los cambios
                        clean_session_cb.set_active(false);
                        will_flag_cb.set_active(false);
                        will_retain_cb.set_active(false);
                        username_flag_cb.set_active(false);
                        password_flag_cb.set_active(false);
                        // ip_entry.set_text("");
                        client_id_entry.set_text("");
                        connect_lwt_entry.set_text("");
                        connect_lwm_entry.set_text("");
                        connect_username_entry.set_text("");
                        connect_password_entry.set_text("");
                        // port_entry.set_text("");
                        connected_entry.set_text("Disconnected");
                        info!("Desconectado correctamente.");
                    }
                    Err(_) => {
                        info!("No se ha podido desconectar.");
                        disconnect_dialog.set_text(Some("Disconnected unsuccessfully!"));
                        disconnect_dialog.set_message_type(gtk::MessageType::Error);
                    }
                }
        }
        disconnect_dialog.show_all()
    }));

    let layout_connect = gtk::Box::new(gtk::Orientation::Vertical, 5);
    layout_connect.add(&con_header);
    layout_connect.add(&ip_layout);
    layout_connect.add(&port_layout);
    layout_connect.add(&save_button_layout);
    layout_connect.add(&client_id_layout);
    layout_connect.add(&clean_s_layout);
    layout_connect.add(&will_flag_layout);
    layout_connect.add(&willqos_layout);
    layout_connect.add(&will_ret_layout);
    layout_connect.add(&username_flag_layout);
    layout_connect.add(&password_flag_layout);
    layout_connect.add(&keep_alive_layout);
    layout_connect.add(&con_pack_layout);
    layout_connect.add(&button_connect);
    layout_connect.add(&button_disconnect);
    layout_connect.add(&connected_layout);

    // PUBLISH
    let pub_header = gtk::HeaderBar::new();
    pub_header.set_title(Some("Publish"));

    let publish_message_label = gtk::Label::new(Some("Message"));
    let publish_message_view = gtk::TextView::new();
    let publish_message_scroll = gtk::ScrolledWindow::new(NONE_ADJUSTMENT, NONE_ADJUSTMENT);
    publish_message_scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    publish_message_scroll.set_size_request(350, 100);
    publish_message_scroll.set_hexpand(true);
    publish_message_scroll.add(&publish_message_view);
    let publish_msg_layout = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    publish_msg_layout.add(&publish_message_label);
    publish_msg_layout.add(&publish_message_scroll);

    let publish_topic_label = gtk::Label::new(Some("Topic"));
    let publish_topic_entry = gtk::Entry::new();
    let publish_topic_layout = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    publish_topic_layout.add(&publish_topic_label);
    publish_topic_layout.add(&publish_topic_entry);

    let qos0_label = gtk::Label::new(Some("QoS 0"));
    let qos1_label = gtk::Label::new(Some("QoS 1"));
    let qos_switch = gtk::Switch::new();
    qos_switch.set_hexpand(false);
    qos_switch.set_size_request(20, 30);
    let qos_layout = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    qos_layout.add(&qos0_label);
    qos_layout.add(&qos_switch);
    qos_layout.add(&qos1_label);

    let retain_label = gtk::Label::new(Some("Retain Message"));
    let retain_cb = gtk::CheckButton::new();
    let retain_layout = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    retain_layout.add(&retain_label);
    retain_layout.add(&retain_cb);

    let publish_button = gtk::Button::with_label("Publish");
    let publish_dialog = gtk::MessageDialog::new(
        None::<&gtk::Window>,
        gtk::DialogFlags::DESTROY_WITH_PARENT,
        gtk::MessageType::Info,
        ButtonsType::Ok,
        "Published unsuccesfully!",
    );

    publish_dialog.connect_button_press_event(|publish_dialog, _| {
        publish_dialog.hide();
        gtk::Inhibit(true)
    });
    // publish_dialog.set_secondary_text(Some("Never received puback"));
    let write_publish = write.clone();
    let _read_publish = read.clone();
    publish_button.connect_clicked(glib::clone!(@weak publish_dialog, @weak qos_switch,
                                                   @weak retain_cb, @weak publish_topic_entry,
                                                   @weak publish_message_view, @weak connected_entry  => move |_| {
        if connected_entry.text() == *"Disconnected"{
            publish_dialog.set_text(Some("No estas conectado!"));
            publish_dialog.set_message_type(gtk::MessageType::Warning);
            publish_dialog.show_all();
            return;
        }
        // Como no implementamos QoS2 el primer bit va a ser siempre 0.
        let qos_b1 = false;
        let mut qos_b2 = false;

        if qos_switch.is_active()
        {
            qos_b2 = true;
        }

        let flags = serializer::new_publish_packet_flags(Some(retain_cb.is_active()), Some(qos_b2),
                                                         Some(qos_b1), Some(false)).ok().unwrap();

        let topic_message = if publish_topic_entry.text().is_empty() { "".to_string() } else { publish_topic_entry.text().to_string() };
        let topic = serializer::new_topic_filter(topic_message).ok().unwrap();

         let buffer = publish_message_view.buffer().unwrap();
         let (start, end) = buffer.bounds();
         let message = if buffer.text(&start, &end, true).unwrap().is_empty() { "".to_string() } else { buffer.text(&start, &end, true).unwrap().to_string() };
         if message != *"" && ! publish_topic_entry.text().is_empty() {
            let result = packets::send_publish(&mut write_publish.lock().unwrap(), flags, topic, message);

            match result {
                Ok(_) => {
                        info!("Paquete PUBLISH enviado correctamente.");
                        if qos_switch.is_active() {
                            // Se quedara esperando a que llegue un mensaje de PUBACK en el read.
                        } else {
                            publish_dialog.set_text(Some("Published Succesfully!"));
                            publish_dialog.set_message_type(gtk::MessageType::Info);
                            publish_dialog.show_all();
                        }
                    },
                Err(mqtt) => {
                    error!("Error al enviar publish: {:?}", mqtt);
                    publish_dialog.set_text(Some("Error al enviar PUBLISH"));
                    publish_dialog.set_message_type(gtk::MessageType::Error);
                    publish_dialog.show_all();
                }
            }
            // publish_dialog.show_all();

            publish_topic_entry.set_text("");
            buffer.set_text("");
            qos_switch.set_active(false);
            retain_cb.set_active(false);
         } else {
            publish_dialog.set_text(Some("Los campos Message y Topic son obligatorios."));
            publish_dialog.set_message_type(gtk::MessageType::Error);
            publish_dialog.show_all();
         }

    }));

    let layout_publish = gtk::Box::new(gtk::Orientation::Vertical, 5);
    layout_publish.add(&pub_header);
    layout_publish.add(&publish_msg_layout);
    layout_publish.add(&publish_topic_layout);
    layout_publish.add(&qos_layout);
    layout_publish.add(&retain_layout);
    layout_publish.add(&publish_button);

    // SUBSCRIBE
    let sub_header = gtk::HeaderBar::new();
    sub_header.set_title(Some("Subscription"));

    let sub_qos0_label = gtk::Label::new(Some("QoS 0"));
    let sub_qos1_label = gtk::Label::new(Some("QoS 1"));
    let sub_qos_switch = gtk::Switch::new();
    sub_qos_switch.set_hexpand(false);
    sub_qos_switch.set_size_request(20, 30);
    let sub_qos_layout = gtk::Box::new(gtk::Orientation::Horizontal, 5);
    sub_qos_layout.add(&sub_qos0_label);
    sub_qos_layout.add(&sub_qos_switch);
    sub_qos_layout.add(&sub_qos1_label);

    let sub_topic_label = gtk::Label::new(Some(
        "Subscribe to topics\n use commas (,) to separate them",
    ));
    let sub_topic_view = gtk::TextView::new();
    sub_topic_view.set_size_request(350, 100);
    let write_sub = write.clone();
    let _read_sub = read.clone();
    let sub_button = gtk::Button::with_label("Subscribe");
    let subscribe_dialog = gtk::MessageDialog::new(
        None::<&gtk::Window>,
        gtk::DialogFlags::DESTROY_WITH_PARENT,
        gtk::MessageType::Error,
        ButtonsType::Ok,
        "Subscribed successfully!",
    );

    subscribe_dialog.connect_button_press_event(|subscribe_dialog, _| {
        subscribe_dialog.hide();
        gtk::Inhibit(true)
    });
    sub_button.connect_clicked(glib::clone!(@weak subscribe_dialog, @weak sub_topic_view, @weak sub_qos_switch, @weak connected_entry => move |_|  {
            if connected_entry.text() == *"Disconnected"{
                subscribe_dialog.set_text(Some("No estas conectado!"));
                subscribe_dialog.set_message_type(gtk::MessageType::Warning);
                subscribe_dialog.show_all();
                return;
            }
            let buffer = sub_topic_view.buffer().unwrap();
            let (start, end) = buffer.bounds();
            let topic = if buffer.text(&start, &end, true).unwrap().is_empty() { "".to_string() } else { buffer.text(&start, &end, true).unwrap().to_string() };
            if topic != *"" {
                let topic_str = topic.split(',').map(|s| s.to_string()).collect();
                if sub_qos_switch.is_active() {
                    match packets::send_subscribe(&mut write_sub.lock().unwrap(), topic_str, 1) {
                        Ok(_) => {
                            subscribe_dialog.set_text(Some("Suscripcion exitosa."));
                            subscribe_dialog.set_message_type(gtk::MessageType::Info);
                            subscribe_dialog.show_all();
                        }, Err(_) => {
                            subscribe_dialog.set_text(Some("Error al suscribirse."));
                            subscribe_dialog.set_message_type(gtk::MessageType::Error);
                            subscribe_dialog.show_all();
                        }
                    }
                } else {
                    match packets::send_subscribe(&mut write_sub.lock().unwrap(), topic_str, 0) {
                        Ok(_) => {
                            subscribe_dialog.set_text(Some("Suscripcion exitosa."));
                            subscribe_dialog.set_message_type(gtk::MessageType::Info);
                            subscribe_dialog.show_all();
                        }, Err(_) => {
                            subscribe_dialog.set_text(Some("Error al suscribirse."));
                            subscribe_dialog.set_message_type(gtk::MessageType::Error);
                            subscribe_dialog.show_all();
                        }
                    }
                }
            }
            else {
                subscribe_dialog.set_text(Some("Completar Topic al cual desea suscribirse."));
                subscribe_dialog.set_message_type(gtk::MessageType::Warning);
                subscribe_dialog.show_all();
            }

        })
    );

    let sub_sub_layout = gtk::Box::new(gtk::Orientation::Vertical, 5);
    sub_sub_layout.add(&sub_qos_layout);
    sub_sub_layout.add(&sub_topic_label);
    sub_sub_layout.add(&sub_topic_view);
    sub_sub_layout.add(&sub_button);
    let write_unsub = write.clone();
    let _read_unsub = read.clone();
    let unsub_topic_label = gtk::Label::new(Some(
        "Unsubscribe from topics \n user commas (,) to separate them",
    ));
    let unsub_topic_view = gtk::TextView::new();
    unsub_topic_view.set_size_request(350, 100);
    let unsub_button = gtk::Button::with_label("Unsubscribe");
    let unsubscribe_dialog = gtk::MessageDialog::new(
        None::<&gtk::Window>,
        gtk::DialogFlags::DESTROY_WITH_PARENT,
        gtk::MessageType::Error,
        ButtonsType::Ok,
        "Unsubscribed successfully!",
    );

    unsubscribe_dialog.connect_button_press_event(|unsubscribe_dialog, _| {
        unsubscribe_dialog.hide();
        gtk::Inhibit(true)
    });
    //unsubscribe_dialog.set_secondary_text(Some("You are no longer subscribed to the topic."));
    unsub_button.connect_clicked(
        glib::clone!(@weak unsubscribe_dialog, @weak unsub_topic_view, @weak connected_entry => move |_| {
            if connected_entry.text() == *"Disconnected"{
                unsubscribe_dialog.set_text(Some("No estas conectado!"));
                unsubscribe_dialog.set_message_type(gtk::MessageType::Warning);
                unsubscribe_dialog.show_all();
                return;
            }
            let buffer = unsub_topic_view.buffer().unwrap();
            let (start, end) = buffer.bounds();
            let topic = if buffer.text(&start, &end, true).unwrap().is_empty() { "".to_string() } else { buffer.text(&start, &end, true).unwrap().to_string() };
            if topic != *"" {
                let topic_str = topic.split(',').map(|s| s.to_string()).collect();
                match packets::send_unsubscribe(&mut write_unsub.lock().unwrap(), topic_str) {
                    Ok(_) => {
                        unsubscribe_dialog.set_text(Some("Cancelación de suscripción exitosa."));
                        unsubscribe_dialog.set_message_type(gtk::MessageType::Info);
                        unsubscribe_dialog.show_all();
                    }, Err(_) => {
                        unsubscribe_dialog.set_text(Some("Error al cancelar suscripción."));
                        unsubscribe_dialog.set_message_type(gtk::MessageType::Error);
                        unsubscribe_dialog.show_all();
                    }
                }
            }
            else {
                unsubscribe_dialog.set_text(Some("Completar Topic del cual desea cancelar la suscripción."));
                unsubscribe_dialog.set_message_type(gtk::MessageType::Warning);
                unsubscribe_dialog.show_all();
            }
        })
    );

    let unsub_sub_layout = gtk::Box::new(gtk::Orientation::Vertical, 5);
    unsub_sub_layout.add(&unsub_topic_label);
    unsub_sub_layout.add(&unsub_topic_view);
    unsub_sub_layout.add(&unsub_button);

    let received_msg_label = gtk::Label::new(Some("Received Messages"));
    let msg_view = gtk::TextView::new();
    let scroll = gtk::ScrolledWindow::new(NONE_ADJUSTMENT, NONE_ADJUSTMENT);
    scroll.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    scroll.set_expand(true);
    scroll.add(&msg_view);

    let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    // El cliente sera el que va a estar escuchando al servidor.
    // Cuando recibe un mensaje, lo enviara por el channel al buffer de la aplicacion.
    let read_subs = read;
    let write_subss = write;

    thread::spawn(move || loop {
        Client::await_packets(&mut read_subs.lock().unwrap(), &tx, &write_subss);
    });

    let msg_buffer = msg_view
        .buffer()
        .expect("Couldn't get buffer from text_view");
    rx.attach(None, move |msg| {
        let split: Vec<&str> = msg.split('|').collect();

        match split[0] {
            "PUBLISH" => {
                let (start, end) = msg_buffer.bounds();
                let mut actual_text = msg_buffer.text(&start, &end, true).unwrap().to_string();
                let new_line = "\n";
                actual_text.push_str(new_line);
                actual_text.push_str(split[1]);
                msg_buffer.set_text(&actual_text);
                //Ahora hago que el scroll este al final siempre.
                msg_view.scroll_to_mark(&msg_buffer.get_insert().unwrap(), 0.0, true, 0.5, 0.5);
            }
            "CONNACK" => {
                if split[1].eq("Conexion aceptada!") {
                    connected_entry.set_text("Connected");
                    connect_dialog.set_message_type(gtk::MessageType::Info);
                } else {
                    connected_entry.set_text("Disconnected");
                    connect_dialog.set_message_type(gtk::MessageType::Error);
                }

                connect_dialog.set_text(Some(split[1]));
                connect_dialog.show_all()
            }
            "PUBACK" => {
                publish_dialog.set_text(Some("Published Succesfully!"));
                publish_dialog.set_message_type(gtk::MessageType::Info);
                publish_dialog.show_all();
            }
            "SUBACK" => {
                if split[1].starts_with("Subscribed Succesfully") {
                    subscribe_dialog.set_message_type(gtk::MessageType::Info);
                } else {
                    subscribe_dialog.set_message_type(gtk::MessageType::Error);
                }
                subscribe_dialog.set_text(Some(split[1]));
                subscribe_dialog.show_all();
            }
            "UNSUBACK" => {
                if split[1].starts_with("Unubscribed Succesfully") {
                    unsubscribe_dialog.set_message_type(gtk::MessageType::Info);
                } else {
                    unsubscribe_dialog.set_message_type(gtk::MessageType::Error);
                }
                unsubscribe_dialog.set_text(Some(split[1]));
                unsubscribe_dialog.show_all();
            }
            _ => {}
        }
        glib::Continue(true)
    });

    let layout_subscribe = gtk::Box::new(gtk::Orientation::Vertical, 5);
    layout_subscribe.add(&sub_header);
    layout_subscribe.add(&sub_sub_layout);
    layout_subscribe.add(&unsub_sub_layout);
    layout_subscribe.add(&received_msg_label);
    layout_subscribe.add(&scroll);

    // MENU ITEMS
    let menu_bar = gtk::MenuBar::new();
    let connect_menu_item = gtk::MenuItem::with_label("Connection");
    let publish_menu_item = gtk::MenuItem::with_label("Publish");
    let subscribe_menu_item = gtk::MenuItem::with_label("Subscription");
    let quit = gtk::MenuItem::with_label("Quit");

    menu_bar.append(&connect_menu_item);
    menu_bar.append(&publish_menu_item);
    menu_bar.append(&subscribe_menu_item);
    menu_bar.append(&quit);

    // PANTALLA GENERAL
    let layout_general = gtk::Box::new(gtk::Orientation::Vertical, 5);
    layout_general.add(&menu_bar);

    layout_general.add(&layout_connect);
    layout_general.add(&layout_publish);
    layout_general.add(&layout_subscribe);

    window.add(&layout_general);
    let socket = socket.try_clone().unwrap();
    // ACCIONES DE NAVEGACION
    // gblib::clone! will automatically create the new reference and pass it with the same name into the closure.
    quit.connect_activate(glib::clone!(@weak window => move |_| {
        let _res = socket.shutdown(Shutdown::Both);
        window.close();
    }));

    connect_menu_item.connect_activate(glib::clone!(@weak layout_connect, @weak layout_subscribe, @weak layout_publish => move |_| {
        layout_connect.show();
        layout_publish.hide();
        layout_subscribe.hide();
    }));

    publish_menu_item.connect_activate(glib::clone!(@weak layout_connect, @weak layout_subscribe, @weak layout_publish => move |_| {
        layout_connect.hide();
        layout_publish.show();
        layout_subscribe.hide();
    }));

    subscribe_menu_item.connect_activate(
        glib::clone!(@weak layout_connect, @weak layout_subscribe, @weak layout_publish =>move |_| {
            layout_connect.hide();
            layout_publish.hide();
            layout_subscribe.show();
        }),
    );

    window.show_all();

    // Cuando entramos por primera vez queremos ver solo connect
    layout_publish.hide();
    layout_subscribe.hide();
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_sample_client() {
        assert_eq!(1, 1)
    }
}
