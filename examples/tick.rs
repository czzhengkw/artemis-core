use anyhow::Result;
use artemis_core::{Collector, CollectorStream, Engine, Executor, Strategy};
use async_trait::async_trait;
use std::time::{Duration, Instant};
use tokio::time;
use tokio_stream::{wrappers::IntervalStream, StreamExt};
use tracing::{info, subscriber::set_global_default};

#[derive(Debug, Clone)]
pub struct Event {
    pub current: Instant,
}

pub struct TickCollector {
    pub interval: Duration,
}

#[async_trait]
impl Collector<Event> for TickCollector {
    async fn get_event_stream(&self) -> Result<CollectorStream<'_, Event>> {
        tracing::info!("get_event_stream");
        let interval = self.interval;
        let stream = IntervalStream::new(time::interval(interval)).map(|_| Event {
            current: Instant::now(),
        });
        tracing::info!("return stream");
        Ok(Box::pin(stream))
    }
}

pub struct TickStrategy {
    pub current: Instant,
}

#[async_trait]
impl Strategy<Event, Action> for TickStrategy {
    /// Sync the initial state of the strategy if needed, usually by fetching
    /// onchain data.
    async fn sync_state(&mut self) -> Result<()> {
        self.current = Instant::now();
        Ok(())
    }

    /// Process an event, and return an action if needed.
    async fn process_event(&mut self, event: Event) -> Vec<Action> {
        tracing::info!("Event: {:?}", event.current);
        vec![Action {
            current: event.current,
        }]
    }
}

#[derive(Debug, Clone)]
pub struct Action {
    pub current: Instant,
}

pub struct TickExecutor {}

#[async_trait]
impl Executor<Action> for TickExecutor {
    async fn execute(&self, action: Action) -> Result<()> {
        tracing::info!("Action: {:?}", action.current);
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .finish();
    set_global_default(subscriber).unwrap();

    let mut engine = Engine::default();
    engine.add_collector(Box::new(TickCollector {
        interval: Duration::from_secs(1),
    }));
    engine.add_executor(Box::new(TickExecutor {}));
    engine.add_strategy(Box::new(TickStrategy {
        current: Instant::now(),
    }));
    if let Ok(mut set) = engine.run().await {
        while let Some(res) = set.join_next().await {
            info!("res: {:?}", res);
        }
    }
}
