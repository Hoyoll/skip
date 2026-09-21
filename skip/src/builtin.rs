use std::marker::PhantomData;

use crate::{Mouse, Proc, Renderer, State, Vec2};

pub(crate) struct ProcArg<'skip, R: Renderer, P: Proc<'skip, R>> {
    pub proc: P,
    pub arg: P::Arg,
    pub ph: PhantomData<&'skip ()>,
}

pub(crate) trait FromRef {
    fn from(&self) -> Self;
}

pub(crate) trait Widget<'skip, R: Renderer> {
    type Constructor;
    //type RenderArg;
    fn inherit<PO: Into<Vec2<f32>>>(pos: PO, constructor: impl AsRef<Self::Constructor>, renderer: R) -> Self;
    fn renderer(self) -> R;
    fn size(&self) -> Vec2<f32>;
    //fn mouse_state(&self) -> &Vec<(Mouse, State)>;
    fn iter_mouse<'a, F: FnMut(&'a (Mouse, State))>(&self, f: F);
}
