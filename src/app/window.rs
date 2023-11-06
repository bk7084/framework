use pyo3::prelude::*;
use winit::window::Fullscreen;

#[pyclass]
#[pyo3(name = "Window")]
#[derive(Debug, Clone)]
pub struct PyWindowBuilder {
    pub size: Option<[u32; 2]>,
    pub position: Option<[u32; 2]>,
    pub resizable: bool,
    pub title: String,
    pub fullscreen: Option<Fullscreen>,
    pub maximized: bool,
    pub transparent: bool,
    pub decorations: bool,
}

impl Default for PyWindowBuilder {
    #[inline]
    fn default() -> Self {
        Self {
            size: None,
            position: None,
            resizable: true,
            title: "BK7084".to_owned(),
            maximized: false,
            fullscreen: None,
            transparent: false,
            decorations: true,
        }
    }
}

#[pymethods]
impl PyWindowBuilder {
    #[new]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the size of the window.
    #[pyo3(signature = (width=800, height=600))]
    pub fn set_size(&mut self, width: u32, height: u32) {
        self.size = Some([width, height]);
    }

    /// Set the position of the window.
    #[pyo3(signature = (x=200, y=200))]
    pub fn set_position(&mut self, x: u32, y: u32) {
        self.position = Some([x, y]);
    }

    /// Set whether the window is resizable.
    pub fn set_resizable(&mut self, resizable: bool) {
        self.resizable = resizable;
    }

    /// Set the title of the window.
    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    /// Set whether the window is fullscreen.
    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        self.fullscreen = if fullscreen {
            Some(Fullscreen::Borderless(None))
        } else {
            None
        };
    }

    /// Set whether the window is maximized.
    pub fn set_maximized(&mut self, maximized: bool) {
        self.maximized = maximized;
    }

    /// Set whether the window is transparent.
    pub fn set_transparent(&mut self, transparent: bool) {
        self.transparent = transparent;
    }

    /// Set whether the window has decorations.
    pub fn set_decorations(&mut self, decorations: bool) {
        self.decorations = decorations;
    }
}
