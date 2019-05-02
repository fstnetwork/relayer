use std::time::Duration;

error_chain! {
    foreign_links {
        TimerError(tokio::timer::Error);
    }

    errors {
        InvalidInterval(interval: Duration) {
            description("Invalid interval value")
            display("Invalid interval value: {:?}", interval)
        }
    }
}
