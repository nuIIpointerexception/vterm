use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

use ash::{Entry, ext::debug_utils, khr::surface, vk, vk::Handle};
use log::{debug, warn};
use winit::{
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
};

use vui::{
    errors::{InstanceError, WindowError},
    vulkan::{
        instance::Instance, render_device::RenderDevice,
        window_surface::WindowSurface,
    },
};

use crate::{
    cli::Args, VULKAN_APP_NAME, VULKAN_APP_VERSION, VULKAN_ENGINE_NAME,
    VULKAN_ENGINE_VERSION,
};

fn create_instance(
    window: &Window,
    entry: &Entry,
    args: &Args,
) -> Result<Instance, InstanceError> {
    // maybe sometime someone will optimize their stuff to this app...
    let app_name = CString::new(VULKAN_APP_NAME).unwrap();
    let app_version = vk::make_api_version(
        0,
        VULKAN_APP_VERSION.0,
        VULKAN_APP_VERSION.1,
        VULKAN_APP_VERSION.2,
    );
    let engine_name = CString::new(VULKAN_ENGINE_NAME).unwrap();
    let engine_version = vk::make_api_version(
        0,
        VULKAN_ENGINE_VERSION.0,
        VULKAN_ENGINE_VERSION.1,
        VULKAN_ENGINE_VERSION.2,
    );
    let app_info = vk::ApplicationInfo::default()
        .application_name(&app_name)
        .application_version(app_version)
        .engine_name(&engine_name)
        .engine_version(engine_version)
        .api_version(vk::API_VERSION_1_2);

    let layers =
        unsafe { entry.enumerate_instance_layer_properties() }.unwrap();
    let mut layer_names = Vec::new();

    // TODO(nuii): disable in prod.
    let validation = if !args.disable_validation {
        if let Some(layer) = find_layer(&layers, "VK_LAYER_KHRONOS_validation")
        {
            layer_names.push(layer);
            true
        } else {
            warn!("vulkan validation layers not available");
            false
        }
    } else {
        debug!("vulkan validation layers disabled");
        false
    };

    // we only need a layer for debug atm.
    let mut extension_names = ash_window::enumerate_required_extensions(
        window.display_handle().unwrap().as_raw(),
    )
    .unwrap()
    .to_vec();
    extension_names.push(debug_utils::NAME.as_ptr());

    let mut instance_create_info = vk::InstanceCreateInfo::default()
        .application_info(&app_info)
        .enabled_layer_names(&layer_names)
        .enabled_extension_names(&extension_names);

    let validation_feature_enables;
    let mut validation_features;
    if validation {
        validation_feature_enables =
            [vk::ValidationFeatureEnableEXT::DEBUG_PRINTF];
        validation_features = vk::ValidationFeaturesEXT::default()
            .enabled_validation_features(&validation_feature_enables);
        instance_create_info =
            instance_create_info.push_next(&mut validation_features);
    }

    let instance =
        unsafe { entry.create_instance(&instance_create_info, None) }
            .map_err(InstanceError::UnableToCreateInstance);

    Instance::new(instance.unwrap(), entry)
}

fn vulkan_str(slice: &[c_char; 256]) -> &str {
    unsafe { CStr::from_ptr(slice.as_ptr()) }.to_str().unwrap()
}

fn find_layer(layers: &[vk::LayerProperties], name: &str) -> Option<*const i8> {
    for layer in layers {
        if vulkan_str(&layer.layer_name) == name {
            return Some(layer.layer_name.as_ptr());
        }
    }
    None
}

fn create_surface(
    window: &Window,
    entry: &Entry,
    instance: &Instance,
) -> WindowSurface {
    let handle = unsafe {
        ash_window::create_surface(
            entry,
            &instance.ash,
            window.display_handle().unwrap().as_raw(),
            window.window_handle().unwrap().as_raw(),
            None,
        )
    }
    .unwrap();

    WindowSurface::new(
        vk::SurfaceKHR::from_raw(handle.as_raw()),
        surface::Instance::new(entry, &instance.ash),
    )
}

pub fn create_vulkan_device(
    window: &Window,
    entry: Entry,
    args: &Args,
) -> Result<RenderDevice, WindowError> {
    let instance = create_instance(window, &entry, args).unwrap();
    let surface = create_surface(window, &entry, &instance);

    let device = RenderDevice::new(instance, surface)
        .map_err(WindowError::UnexpectedRenderDeviceError)?;

    let (w, h): (u32, u32) = window.inner_size().into();

    device
        .rebuild_swapchain((w, h))
        .expect("Unable to rebuild swapchain");

    Ok(device)
}
