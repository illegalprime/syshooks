extern crate alsa_sys as alsa;

use std::ptr;
use std::ffi::CString;

use self::alsa::{
    snd_mixer_t,
    snd_mixer_selem_id_t,
    snd_mixer_open,
    snd_mixer_attach,
    snd_mixer_selem_register,
    snd_mixer_load,
    snd_mixer_selem_id_malloc,
    snd_mixer_selem_id_set_index,
    snd_mixer_selem_id_set_name,
    snd_mixer_find_selem,
    snd_mixer_close,
    snd_mixer_elem_t,
    snd_mixer_selem_id_free,
    snd_mixer_selem_get_playback_volume_range,
    snd_mixer_selem_set_playback_volume_all,
    snd_mixer_selem_is_playback_mono,
    snd_mixer_selem_set_playback_switch_all,
    snd_mixer_selem_has_playback_switch,
    snd_mixer_selem_get_playback_switch,
    snd_mixer_selem_get_playback_volume,
    SND_MIXER_SCHN_MONO,
};

pub struct Mixer {
    handle: *mut snd_mixer_t,
    elem: *mut snd_mixer_elem_t,
}

impl Mixer {
    pub fn new(card: &str, name: &str) -> Result<Self, AlsaError> {

        let card = match CString::new(card) {
            Ok(c) => c,
            Err(_) => return Err(AlsaError::CardStrContainsNull),
        };
        let name = match CString::new(name) {
            Ok(n) => n,
            Err(_) => return Err(AlsaError::NameStrContainsNull),
        };

        // Load the handle
        let mut handle: *mut snd_mixer_t = ptr::null_mut();
        unsafe {
            if snd_mixer_open(&mut handle, 0) != 0 {
                return Err(AlsaError::MixerOpen);
            }
            if snd_mixer_attach(handle, card.as_ptr()) != 0 {
                return Err(AlsaError::MixerAttach);
            }
            if snd_mixer_selem_register(handle, ptr::null_mut(), ptr::null_mut()) != 0 {
                return Err(AlsaError::MixerRegister);
            }
            if snd_mixer_load(handle) != 0 {
                return Err(AlsaError::MixerLoad);
            }
        }

        // Find the element
        let mut id: *mut snd_mixer_selem_id_t = ptr::null_mut();
        let element = unsafe {
            snd_mixer_selem_id_malloc(&mut id);
            snd_mixer_selem_id_set_index(id, 0);
            snd_mixer_selem_id_set_name(id, name.as_ptr());
            snd_mixer_find_selem(handle, id)
        };

        if element.is_null() {
            return Err(AlsaError::MixerFindSelem);
        }

        // Clean up the ID
        unsafe {
            snd_mixer_selem_id_free(id);
        }

        Ok(Mixer {
            handle: handle,
            elem: element,
        })
    }

    pub fn volume_range(&self) -> (i64, i64) {
        let min: *mut i64 = &mut 0;
        let max: *mut i64 = &mut 0;
        unsafe {
            snd_mixer_selem_get_playback_volume_range(self.elem, min, max);
        }
        assert!(!min.is_null());
        assert!(!max.is_null());
        unsafe {
            (*min, *max)
        }
    }

    pub fn set_volume_raw(&self, volume: i64) {
        unsafe {
            snd_mixer_selem_set_playback_volume_all(self.elem, volume);
        }
    }

    fn vol_raw_to_perc(&self, raw: i64) -> (f32, i64, i64) {
        let (min, max) = self.volume_range();
        ((raw - min) as f32 / (max - min) as f32, min, max)
    }

    fn vol_perc_to_raw(&self, perc: f32) -> (i64, i64, i64) {
        let (min, max) = self.volume_range();
        (((max - min) as f32 * perc + min as f32) as i64, min, max)
    }

    pub fn set_volume(&self, volume: f32) {
        let (vol, _, _) = self.vol_perc_to_raw(volume);
        self.set_volume_raw(vol as i64);
    }

    pub fn volume_raw(&self) -> i64 {
        let vol: *mut i64 = &mut 0;
        unsafe {
            snd_mixer_selem_get_playback_volume(self.elem, SND_MIXER_SCHN_MONO, vol);
            *vol
        }
    }

    pub fn volume(&self) -> f32 {
        let vol = self.volume_raw();
        self.vol_raw_to_perc(vol).0
    }

    pub fn change_volume_raw(&self, delta: i64) {
        let vol = self.volume_raw();
        self.set_volume_raw(vol + delta);
    }

    pub fn change_volume(&self, delta: f32) {
        let (del, _, _) = self.vol_perc_to_raw(delta);
        self.change_volume_raw(del);
    }

    pub fn change_volume_clip(&self, delta: f32) {
        let (del, min, max) = self.vol_perc_to_raw(delta);
        let curr = self.volume_raw();
        let vol = if del + curr > max {
            max
        } else if del + curr < min {
            min
        } else {
            curr + del
        };
        self.set_volume_raw(vol);
    }

    pub fn is_mono(&self) -> bool {
        unsafe {
            snd_mixer_selem_is_playback_mono(self.elem) != 0
        }
    }

    pub fn can_mute(&self) -> bool {
        unsafe {
            snd_mixer_selem_has_playback_switch(self.elem) != 0
        }
    }

    pub fn mute(&self) -> Result<(), ()> {
        if self.can_mute() {
            self.raw_mute();
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn unmute(&self) -> Result<(), ()> {
        if self.can_mute() {
            self.raw_unmute();
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn is_muted(&self) -> bool {
        if self.can_mute() {
            self.raw_is_muted()
        } else {
            false
        }
    }

    pub fn toggle_mute(&self) -> Result<(), ()> {
        if self.can_mute() {
            if self.raw_is_muted() {
                self.raw_unmute()
            } else {
                self.raw_mute()
            }
            Ok(())
        } else {
            Err(())
        }
    }

    fn raw_is_muted(&self) -> bool {
        let is_muted: *mut i32 = &mut 0;
        unsafe {
            snd_mixer_selem_get_playback_switch(self.elem, SND_MIXER_SCHN_MONO, is_muted);
            *is_muted == 0
        }
    }

    fn raw_unmute(&self) {
        unsafe {
            snd_mixer_selem_set_playback_switch_all(self.elem, 1);
        }
    }

    fn raw_mute(&self) {
        unsafe {
            snd_mixer_selem_set_playback_switch_all(self.elem, 0);
        }
    }
}

impl Drop for Mixer {
    fn drop(&mut self) {
        unsafe {
            snd_mixer_close(self.handle);
        }
    }
}

#[derive(Debug)]
pub enum AlsaError {
    NameStrContainsNull,
    CardStrContainsNull,
    MixerOpen,
    MixerAttach,
    MixerLoad,
    MixerRegister,
    MixerFindSelem,
}
