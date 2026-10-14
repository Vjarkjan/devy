#![cfg_attr(target_vendor = "nintendo", no_std)]
#![cfg_attr(target_vendor = "nintendo", no_main)]

use devy::prelude::*;

fn setup(mut video: NonSendMut<Video>) {
    video.show_console(true);
    video.write_console(c"A: guardar en fat:/devy/demo.bin\nB: leer el archivo\n");
}

fn files(
    buttons: Res<ButtonInput<DsButton>>,
    mut storage: ResMut<Storage>,
    mut video: NonSendMut<Video>,
) {
    if buttons.just_pressed(DsButton::A) {
        let result = storage
            .create_dir(c"fat:/devy")
            .and_then(|()| storage.write(c"fat:/devy/demo.bin", b"devy 0.1"));
        video.write_console(if result.is_ok() {
            c"Guardado\n"
        } else {
            c"Error al guardar\n"
        });
    }
    if buttons.just_pressed(DsButton::B) {
        let mut buffer = [0; 32];
        let result = storage.read_into(c"fat:/devy/demo.bin", &mut buffer);
        video.write_console(if result.is_ok() {
            c"Leido\n"
        } else {
            c"Error al leer\n"
        });
    }
}

fn application() -> App {
    let mut app = App::new();
    app.add_plugins((DsPlugins, StoragePlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, files);
    app
}

#[cfg(target_vendor = "nintendo")]
#[unsafe(no_mangle)]
pub extern "C" fn main() -> core::ffi::c_int {
    devy::run(application())
}

#[cfg(not(target_vendor = "nintendo"))]
fn main() {
    let mut app = application();
    app.update();
    assert_eq!(
        app.world_mut().resource_mut::<Storage>().mount(),
        Err(StorageError::Unsupported)
    );
    println!("El almacenamiento FAT necesita Nintendo DS; no se escribieron archivos del host.");
}
