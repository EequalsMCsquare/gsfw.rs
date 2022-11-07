pub mod chanrpc;
pub mod codec;
pub mod component;
pub mod error;
pub mod gs;
pub mod network;

#[cfg(test)]
mod test {
    use async_trait::async_trait;
    use tokio::sync::mpsc;

    use crate::{
        chanrpc::{self, broker, Proto},
        component::{Component, ComponentBuilder},
        gs,
    };

    type ChanCtx = chanrpc::ChanCtx<ChanProto, ModuleName, Box<dyn std::error::Error + Send>>;

    struct Broker {
        name: ModuleName,
        foo: mpsc::Sender<ChanCtx>,
    }

    impl
        broker::Broker<
            ChanProto,
            ModuleName,
            Box<dyn std::error::Error + Send>,
            mpsc::Sender<ChanCtx>,
            mpsc::Receiver<ChanCtx>,
        > for Broker
    {
        fn new(
            name: ModuleName,
            tx_map: &std::collections::HashMap<ModuleName, mpsc::Sender<ChanCtx>>,
        ) -> Self {
            Self {
                name,
                foo: tx_map.get(&ModuleName::Foo).unwrap().clone(),
            }
        }

        fn name(&self) -> ModuleName {
            self.name.clone()
        }

        fn tx(&self, name: ModuleName) -> &mpsc::Sender<ChanCtx> {
            match name {
                ModuleName::Foo => &self.foo,
            }
        }

        fn channel(size: usize) -> (mpsc::Sender<ChanCtx>, mpsc::Receiver<ChanCtx>) {
            mpsc::channel(size)
        }
    }

    #[derive(Debug, PartialEq, Eq, Hash, Clone)]
    enum ModuleName {
        Foo,
    }

    enum ChanProto {
        CtrlShutdown,
    }

    impl Proto for ChanProto {
        fn proto_shutdown() -> Self {
            Self::CtrlShutdown
        }
    }

    struct FooComponent {
        rx: mpsc::Receiver<ChanCtx>,
    }

    #[async_trait]
    impl Component<ChanProto, ModuleName, Box<dyn std::error::Error + Send>> for FooComponent {
        fn name(&self) -> ModuleName {
            ModuleName::Foo
        }
        async fn init(&mut self) -> Result<(), Box<dyn std::error::Error + Send>> {
            Ok(())
        }

        async fn run(mut self: Box<Self>) -> Result<(), Box<dyn std::error::Error + Send>> {
            loop {
                if let Some(msg) = self.rx.recv().await {
                    match msg.payload {
                        ChanProto::CtrlShutdown => return Ok(()),
                    }
                } else {
                    return Ok(());
                }
            }
        }
    }

    #[derive(Default)]
    struct FooComponentBuilder {
        broker: Option<Broker>,
        rx: Option<mpsc::Receiver<ChanCtx>>,
    }

    impl
        ComponentBuilder<
            ChanProto,
            ModuleName,
            Broker,
            Box<dyn std::error::Error + Send>,
            mpsc::Receiver<ChanCtx>,
        > for FooComponentBuilder
    {
        fn build(
            self: Box<Self>,
        ) -> Box<
            dyn crate::component::Component<
                ChanProto,
                ModuleName,
                Box<dyn std::error::Error + Send>,
            >,
        > {
            Box::new(FooComponent {
                rx: self.rx.unwrap(),
            })
        }

        fn name(&self) -> ModuleName {
            ModuleName::Foo
        }

        fn set_rx(&mut self, rx: mpsc::Receiver<ChanCtx>) {
            self.rx = Some(rx);
        }

        fn set_broker(&mut self, broker: Broker) {
            self.broker = Some(broker)
        }
    }

    #[tokio::test]
    async fn foo() -> Result<(), Box<dyn std::error::Error>> {
        gs::GameBuilder::new()
            .component(FooComponentBuilder::default())
            .serve()?
            .await;
        Ok(())
    }
}
