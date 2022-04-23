use crossbeam::channel::{Receiver, Sender};

use super::messages::{RenderTask, SubRenderResult, ToMasterMsg, ToWorkerMsg};

type Channel<T> = (Sender<T>, Receiver<T>);

#[derive(Clone)]
pub struct CompWorkerComms {
    // Comp x renderer comms
    pub task_sen: Sender<RenderTask>,
    pub result_rec: Receiver<SubRenderResult>,
    // Comp x master comms
    pub master_sen: Sender<ToMasterMsg>,
    pub command_rec: Receiver<ToWorkerMsg>,
}

#[derive(Clone)]
pub struct RenderWorkerComms {
    // Comp x renderer comms
    pub result_sen: Sender<SubRenderResult>,
    pub task_rec: Receiver<RenderTask>,
    // Comp x master comms
    pub command_rec: Receiver<ToWorkerMsg>,
}

#[derive(Clone)]
pub struct MasterComms {
    pub result_receiver: Receiver<ToMasterMsg>,
    pub command_sender: Vec<Sender<ToWorkerMsg>>,
}

#[derive(Clone)]
pub struct CommsBuilder {
    ren_to_comp: Channel<SubRenderResult>, // Render -> Comp
    comp_to_ren: Channel<RenderTask>,      // Comp -> Render
    // Command channels are in order by ID, Renderers first
    command: Vec<Channel<ToWorkerMsg>>, // Master -> Worker
    results: Channel<ToMasterMsg>,      // Comp -> Master
}

impl CommsBuilder {
    pub fn new(n_of_workers: usize) -> CommsBuilder {
        let command: Vec<_> = std::iter::repeat_with(|| crossbeam::channel::bounded(10000))
            .take(n_of_workers)
            .collect();

        let ren_to_comp = crossbeam::channel::bounded(10000);
        let comp_to_ren = crossbeam::channel::bounded(10000);
        let results = crossbeam::channel::bounded(10000);

        CommsBuilder {
            ren_to_comp,
            comp_to_ren,
            command,
            results,
        }
    }

    pub fn renderer(&self, id: usize) -> RenderWorkerComms {
        let result_sen = self.ren_to_comp.0.clone();
        let task_rec = self.comp_to_ren.1.clone();
        let command_rec = self.command[id].1.clone();

        RenderWorkerComms {
            result_sen,
            task_rec,
            command_rec,
        }
    }

    pub fn compositor(&self, id: usize) -> CompWorkerComms {
        let task_sen = self.comp_to_ren.0.clone();
        let result_rec = self.ren_to_comp.1.clone();
        let master_sen = self.results.0.clone();
        let command_rec = self.command[id].1.clone();

        CompWorkerComms {
            task_sen,
            result_rec,
            master_sen,
            command_rec,
        }
    }

    pub fn master(&self) -> MasterComms {
        let result_receiver = self.results.1.clone();
        let command_sender = self.command.iter().map(|ch| ch.0.clone()).collect();

        MasterComms {
            result_receiver,
            command_sender,
        }
    }
}
