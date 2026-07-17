use std::marker::PhantomData;

use crate::{Proc, Renderer, Vec2};

pub(crate) struct ProcArg<'skip, R: Renderer, P: Proc<'skip, R>> {
    pub proc: P,
    pub arg: P::Arg,
    pub ph: PhantomData<(&'skip ())>,
}

pub(crate) trait Widget<'skip, R: Renderer> {
    fn inherit<P: Into<Vec2<f32>>, PO: Into<Vec2<f32>>>(dim: P, pos: PO, renderer: R) -> Self;
    fn renderer(self) -> R;
    fn size(&mut self) -> Vec2<f32>;
}
