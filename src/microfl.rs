use crate::{
    keylist::MicroKeyList,
    prelude::{ProgressResolution, TimingResolution},
};

pub struct UFrameList<
    'a,
    const N: usize,
    UN: Eq,
    TRES: TimingResolution + Clone,
    PRES: ProgressResolution + Eq,
>(pub [(&'static str, &'a mut [MicroKeyList<'a, UN, TRES, PRES>]); N]);

impl<'a, const N: usize, UN: Eq, TRES: TimingResolution + Clone, PRES: ProgressResolution + Eq>
    UFrameList<'a, N, UN, TRES, PRES>
{
    pub fn get_progress(&mut self, class: &'static str, id: UN) -> PRES {
        for (inclass, ukeylists) in &mut self.0 {
            if *inclass == class {
                for ukeylist in ukeylists.iter_mut() {
                    for (inid, progresslist) in &mut *ukeylist.0 {
                        if *inid == id {
                            return progresslist.get_progress();
                        }
                    }
                }
                break;
            }
        }
        PRES::zero()
    }
    pub fn get_progress_f32(&mut self, class: &'static str, id: UN) -> f32 {
        for (inclass, ukeylists) in &mut self.0 {
            if *inclass == class {
                for ukeylist in ukeylists.iter_mut() {
                    for (inid, progresslist) in &mut *ukeylist.0 {
                        if *inid == id {
                            return progresslist.get_progress_f32();
                        }
                    }
                }
                break;
            }
        }
        PRES::zero().to_f32()
    }
}
