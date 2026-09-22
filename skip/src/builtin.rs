use std::marker::PhantomData;

use crate::{Mouse, Proc, Renderer, State, Vec2};

pub(crate) trait Widget<'skip, R: Renderer> {
    type Constructor: Clone; 
    fn inherit<PO: Into<Vec2<f32>>>(
        pos: PO,
        constructor: Self::Constructor,
        renderer: R,
    ) -> Self;
    fn renderer(self) -> R;
    fn size(&self) -> Vec2<f32>;
    //fn mouse_state(&self) -> &Vec<(Mouse, State)>;
    fn iter_mouse<'a, F: FnMut(&'a (Mouse, State))>(&self, f: F);
}
