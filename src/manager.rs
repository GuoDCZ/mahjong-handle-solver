use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::sync::mpsc::{Receiver, Sender, channel};

pub struct Chain<First, Second>(First, Second);

pub trait Chainable {
    type Input;
    type Output;

    fn run(&self, input: Self::Input) -> Self::Output;

    fn chain<Next>(self, second: Next) -> Chain<Self, Next>
    where
        Self: Chainable + Sized,
        Next: Chainable<Input = Self::Output>,
    {
        Chain(self, second)
    }
}

impl<First, Second> Chainable for Chain<First, Second>
where
    First: Chainable,
    Second: Chainable<Input = First::Output>,
{
    type Input = First::Input;
    type Output = Second::Output;

    fn run(&self, input: Self::Input) -> Self::Output {
        self.1.run(self.0.run(input))
    }
}

pub trait MapWorker {
    type Input;
    type Output;

    fn work(input: Self::Input) -> Self::Output;
}

pub trait ReduceWorker {
    type Input;
    type Output;

    fn new() -> Self;
    fn work(&mut self, input: Self::Input);
    fn term(&mut self) -> Self::Output;
}

pub trait Processing {
    type Input: Send + 'static;
    type Output: Send + 'static;

    fn process(worker_rx: Receiver<Self::Input>, tx: Sender<Self::Output>);
    fn spawn(worker_rx: Receiver<Self::Input>, tx: Sender<Self::Output>) {
        std::thread::spawn(move || {
            Self::process(worker_rx, tx);
        });
    }
}

struct MapProcessor<W>(std::marker::PhantomData<W>);

impl<W: MapWorker> Processing for MapProcessor<W>
where
    W::Input: Send + 'static,
    W::Output: Send + 'static,
{
    type Input = W::Input;
    type Output = W::Output;

    fn process(worker_rx: Receiver<Self::Input>, tx: Sender<Self::Output>) {
        for input in worker_rx {
            tx.send(W::work(input)).unwrap();
        }
    }
}

struct ReduceProcessor<W>(std::marker::PhantomData<W>);

impl<W: ReduceWorker> Processing for ReduceProcessor<W>
where
    W::Input: Send + 'static,
    W::Output: Send + 'static,
{
    type Input = W::Input;
    type Output = W::Output;

    fn process(worker_rx: Receiver<Self::Input>, tx: Sender<Self::Output>) {
        let mut worker = W::new();
        for input in worker_rx {
            worker.work(input);
        }
        tx.send(worker.term()).unwrap();
    }
}

pub trait Manager {
    type P: Processing;
    const N_WORKERS: u64;
    const N_TASKS: u64;
}

impl<P: Processing, M: Manager<P = P>> Chainable for M
where
    P: Processing + Send + 'static,
    P::Input: Send + 'static,
    P::Output: Send + 'static,
{
    type Input = (Receiver<P::Input>, MultiProgress);
    type Output = (Receiver<P::Output>, MultiProgress);

    fn run(&self, input_rx: Self::Input) -> Self::Output {
        let (tx, rx) = channel();
        let (input_rx, mpg) = input_rx;
        let pb = mpg.add(ProgressBar::new(M::N_TASKS));

        // if Self::N_WORKERS == 1 {
        //     P::spawn(input_rx, tx);
        // } else {
        let mut worker_txs: Vec<Sender<P::Input>> = vec![];
        for _ in 0..Self::N_WORKERS {
            let (worker_tx, worker_rx) = channel();
            worker_txs.push(worker_tx.clone());
            let tx_clone = tx.clone();
            P::spawn(worker_rx, tx_clone);
        }
        std::thread::spawn(move || {
            let mut worker_txs = worker_txs.into_iter().cycle();
            for input in input_rx {
                pb.inc(1);
                worker_txs.next().unwrap().send(input).unwrap();
            }
            pb.finish();
        });
        // }
        (rx, mpg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const N_REQUEST: usize = 100000000;

    #[test]
    fn test_manager() {
        let timer = std::time::Instant::now();

        let (tx, rx) = channel();
        for _ in 0..N_REQUEST {
            tx.send(114514).unwrap();
        }

        let mpg = MultiProgress::new();

        let (rx, _) = {
            struct Doubler {}
            impl MapWorker for Doubler {
                type Input = i32;
                type Output = i32;

                fn work(input: Self::Input) -> Self::Output {
                    input * 2
                }
            }

            struct MapDoublerManager {}
            impl Manager for MapDoublerManager {
                type P = MapProcessor<Doubler>;
                const N_WORKERS: u64 = 4;
                const N_TASKS: u64 = N_REQUEST as u64;
            }
            MapDoublerManager {}
        }
        .chain({
            struct Adder {
                sum: i32,
            }
            impl ReduceWorker for Adder {
                type Input = i32;
                type Output = i32;

                fn new() -> Self {
                    Adder { sum: 0 }
                }

                fn work(&mut self, input: Self::Input) {
                    self.sum += input;
                }

                fn term(&mut self) -> Self::Output {
                    self.sum + 2
                }
            }
            struct ReduceAdderManager {}
            impl Manager for ReduceAdderManager {
                type P = ReduceProcessor<Adder>;
                const N_WORKERS: u64 = 4;
                const N_TASKS: u64 = N_REQUEST as u64;
            }
            ReduceAdderManager {}
        })
        .run((rx, mpg));
        for _ in 0..N_REQUEST {
            assert_eq!(rx.recv().unwrap(), 114514 * 2 + 2);
        }

        let elapsed = timer.elapsed();
        println!(
            "Elapsed: {}.{:03} sec",
            elapsed.as_secs(),
            elapsed.subsec_millis()
        );
    }
}

//  Manager<
