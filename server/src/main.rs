use crate::server::Server;
use tracing::info;
extern crate serializer;

mod json_helper;
mod packets;
mod server;
mod socket;

fn start_listening(server: &mut Server) {
    info!("[Server] Servidor comienza a escuchar.");
    server.listen();
}

fn main() {
    // Creo un subscriber no bloqueante para todos los tipos de eventos que escriba en un archivo .log con rotacion diaria.
    let file_appender = tracing_appender::rolling::daily("logs", "server_log.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt().with_writer(non_blocking).init();
    info!("[Server] Inicio de servidor.");
    let mut server = Server::new();
    start_listening(&mut server);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_sample_server() {
        assert_eq!(1, 1)
    }

    #[test]
    fn publish_test() {
        let flags = serializer::new_publish_packet_flags(
            Option::from(true),
            Option::from(true),
            None,
            None,
        )
        .ok()
        .unwrap();
        let topic = serializer::new_topic_filter("topic_sample".to_string())
            .ok()
            .unwrap();
        let flags = flags.hex_value();
        let mut data = vec![(serializer::PacketType::PUBLISH as u8) << 4 | flags, 28];
        data.append(&mut topic.get_filter().clone());
        data.append(&mut "sample message".to_string().into_bytes());
    }
}
