use arrayvec::ArrayVec;
use crossbeam::channel::{Receiver, Sender};

use super::messages::{RenderTask, ToCompositorMsg, ToMasterMsg, ToRendererMsg, ToWorkerMsg};

type Channel<T> = (Sender<T>, Receiver<T>);

pub struct CompWorkerComms<const R: usize> {
    // Comp x renderer comms
    pub renderers: ArrayVec<Sender<ToRendererMsg>, R>,
    pub receiver: Receiver<ToCompositorMsg>,
    // Comp x master comms
    pub result_sender: Sender<ToMasterMsg>,
    pub command_receiver: Receiver<ToWorkerMsg>,
}

impl<const R: usize> CompWorkerComms<R> {
    #[must_use]
    pub fn new(
        renderers: ArrayVec<Sender<ToRendererMsg>, R>,
        receiver: Receiver<ToCompositorMsg>,
        result_sender: Sender<ToMasterMsg>,
        command_receiver: Receiver<ToWorkerMsg>,
    ) -> Self {
        Self {
            renderers,
            receiver,
            result_sender,
            command_receiver,
        }
    }
}

pub struct RenderWorkerComms<const C: usize> {
    // Comp x renderer comms
    pub compositors: ArrayVec<Sender<ToCompositorMsg>, C>,
    pub receiver: Receiver<ToRendererMsg>,
    // Master x renderer comms
    pub task_receiver: Receiver<RenderTask>,
    pub command_receiver: Receiver<ToWorkerMsg>,
}

impl<const C: usize> RenderWorkerComms<C> {
    #[must_use]
    pub fn new(
        compositors: ArrayVec<Sender<ToCompositorMsg>, C>,
        receiver: Receiver<ToRendererMsg>,
        task_receiver: Receiver<RenderTask>,
        command_receiver: Receiver<ToWorkerMsg>,
    ) -> Self {
        Self {
            compositors,
            receiver,
            task_receiver,
            command_receiver,
        }
    }
}

pub struct MasterComms<const RC: usize> {
    pub task_sender: Sender<RenderTask>,
    pub result_receiver: Receiver<ToMasterMsg>,
    pub command_sender: ArrayVec<Sender<ToWorkerMsg>, RC>,
}

impl<const RC: usize> MasterComms<RC> {
    #[must_use]
    pub fn new(
        task_sender: Sender<RenderTask>,
        result_receiver: Receiver<ToMasterMsg>,
        command_sender: ArrayVec<Sender<ToWorkerMsg>, RC>,
    ) -> Self {
        Self {
            task_sender,
            result_receiver,
            command_sender,
        }
    }
}

pub struct CommsBuilder<const R: usize, const C: usize, const RC: usize> {
    ren_to_comp: ArrayVec<Channel<ToCompositorMsg>, C>, // Render -> Comp
    comp_to_ren: ArrayVec<Channel<ToRendererMsg>, R>,   // Comp -> Render
    // Command channels are in order by ID, Renderers first
    command: ArrayVec<Channel<ToWorkerMsg>, RC>, // Master -> Worker
    tasks: Channel<RenderTask>,                  // Master -> Render
    results: Channel<ToMasterMsg>,               // Comp -> Master
}

impl<const R: usize, const C: usize, const RC: usize> CommsBuilder<R, C, RC> {
    pub fn new() -> CommsBuilder<R, C, RC> {
        let ren_to_comp: ArrayVec<_, C> = std::iter::repeat_with(crossbeam::channel::unbounded)
            .take(C)
            .collect();

        let comp_to_ren: ArrayVec<_, R> = std::iter::repeat_with(crossbeam::channel::unbounded)
            .take(R)
            .collect();

        let command: ArrayVec<_, RC> = std::iter::repeat_with(crossbeam::channel::unbounded)
            .take(RC)
            .collect();

        let tasks = crossbeam::channel::unbounded();
        let results = crossbeam::channel::unbounded();

        CommsBuilder {
            ren_to_comp,
            comp_to_ren,
            command,
            tasks,
            results,
        }
    }

    pub fn renderer(&self, id: usize) -> RenderWorkerComms<C> {
        let compositors = self.ren_to_comp.iter().map(|v| v.0.clone()).collect();
        let receiver = self.comp_to_ren[id].1.clone();
        let command_receiver = self.command[id].1.clone();
        let task_receiver = self.tasks.1.clone();

        RenderWorkerComms {
            compositors,
            receiver,
            task_receiver,
            command_receiver,
        }
    }

    pub fn compositor(&self, id: usize) -> CompWorkerComms<R> {
        let renderers = self.comp_to_ren.iter().map(|v| v.0.clone()).collect();
        let receiver = self.ren_to_comp[id].1.clone();
        let command_receiver = self.command[R + id].1.clone();
        let result_sender = self.results.0.clone();

        CompWorkerComms {
            renderers,
            receiver,
            result_sender,
            command_receiver,
        }
    }

    pub fn master(&self) -> MasterComms<RC> {
        let task_sender = self.tasks.0.clone();
        let result_receiver = self.results.1.clone();
        let command_sender = self.command.iter().map(|ch| ch.0.clone()).collect();
        MasterComms {
            task_sender,
            result_receiver,
            command_sender,
        }
    }
}
