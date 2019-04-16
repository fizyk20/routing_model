// Copyright 2019 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{actions::*, flows_dst::*, flows_node::*, flows_src::*, utilities::*};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Default, Clone)]
pub struct ProcessElderChangeState {
    pub is_active: bool,
    pub wait_votes: Vec<ParsecVote>,
    pub change_elder: Option<ChangeElder>,
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct CheckAndProcessElderChangeState {
    pub sub_routine_process_elder_change: ProcessElderChangeState,
}

#[derive(Debug, PartialEq, Clone)]
pub struct AcceptAsCandidateState {
    pub candidate: Candidate,
    pub got_candidate_info: bool,
    pub voted_online: bool,
}

impl AcceptAsCandidateState {
    pub fn new(candidate: Candidate) -> Self {
        Self {
            candidate,
            got_candidate_info: false,
            voted_online: false,
        }
    }
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct StartRelocateSrcState {
    pub already_relocating: BTreeMap<Candidate, i32>,
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct DstRoutineState {
    pub is_processing_candidate: bool,
    pub sub_routine_accept_as_candidate: Option<AcceptAsCandidateState>,
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct SrcRoutineState {}

// The very top level event loop deciding how the sub event loops are processed
#[derive(Debug, PartialEq, Default, Clone)]
pub struct MemberState {
    pub action: Action,
    pub failure: Option<Event>,
    pub dst_routine: DstRoutineState,
    pub src_routine: SrcRoutineState,
    pub start_relocate_src: StartRelocateSrcState,
    pub check_and_process_elder_change_routine: CheckAndProcessElderChangeState,
}

impl MemberState {
    pub fn try_next(&self, event: Event) -> Option<Self> {
        let dst = &self.dst_routine;

        if let Some(next) = self.as_check_and_process_elder_change().try_next(event) {
            return Some(next);
        }

        if self
            .check_and_process_elder_change_routine
            .sub_routine_process_elder_change
            .is_active
        {
            if let Some(next) = self.as_process_elder_change().try_next(event) {
                return Some(next);
            }
        }

        if let Some(next) = self.as_check_online_offline().try_next(event) {
            return Some(next);
        }

        if let Some(next) = self.as_start_relocate_src().try_next(event) {
            return Some(next);
        }

        if let Some(next) = self.as_top_level_src().try_next(event) {
            return Some(next);
        }

        if dst.sub_routine_accept_as_candidate.is_some() {
            if let Some(next) = self.as_accept_as_candidate().try_next(event) {
                return Some(next);
            }
        }

        if let Some(next) = self.as_top_level_dst().try_next(event) {
            return Some(next);
        }

        match event {
            // These should only happen if a routine started them, so it should have
            // handled them too, but other routine are not there yet and we want to test
            // these do not fail.
            Event::ParsecConsensus(ParsecVote::RemoveElderNode(_))
            | Event::ParsecConsensus(ParsecVote::AddElderNode(_))
            | Event::ParsecConsensus(ParsecVote::NewSectionInfo(_)) => Some(self.clone()),

            _ => None,
        }
    }

    pub fn as_top_level_dst(&self) -> TopLevelDst {
        TopLevelDst(self.clone())
    }

    pub fn as_accept_as_candidate(&self) -> AcceptAsCandidate {
        AcceptAsCandidate(self.clone())
    }

    pub fn as_check_and_process_elder_change(&self) -> CheckAndProcessElderChange {
        CheckAndProcessElderChange(self.clone())
    }

    pub fn as_check_online_offline(&self) -> CheckOnlineOffline {
        CheckOnlineOffline(self.clone())
    }

    pub fn as_top_level_src(&self) -> TopLevelSrc {
        TopLevelSrc(self.clone())
    }

    pub fn as_start_relocate_src(&self) -> StartRelocateSrc {
        StartRelocateSrc(self.clone())
    }

    pub fn as_process_elder_change(&self) -> ProcessElderChange {
        ProcessElderChange(self.clone())
    }

    pub fn failure_event(&self, event: Event) -> Self {
        Self {
            failure: Some(event),
            ..self.clone()
        }
    }

    pub fn with_dst_sub_routine_accept_as_candidate(
        &self,
        sub_routine_accept_as_candidate: Option<AcceptAsCandidateState>,
    ) -> Self {
        Self {
            dst_routine: DstRoutineState {
                sub_routine_accept_as_candidate,
                ..self.dst_routine.clone()
            },
            ..self.clone()
        }
    }
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct JoiningRelocateCandidateState {
    pub has_resource_proofs: BTreeMap<Name, (bool, Option<ProofSource>)>,
    pub routine_complete: Option<GenesisPfxInfo /*output*/>,
}

// The very top level event loop deciding how the sub event loops are processed
#[derive(Debug, PartialEq, Default, Clone)]
pub struct JoiningState {
    pub action: Action,
    pub failure: Option<Event>,
    pub join_routine: JoiningRelocateCandidateState,
}

impl JoiningState {
    pub fn start(&self, new_section: SectionInfo) -> Self {
        self.as_joining_relocate_candidate()
            .start_event_loop(new_section)
            .0
    }

    pub fn try_next(&self, event: Event) -> Option<Self> {
        if let Some(next) = self.as_joining_relocate_candidate().try_next(event) {
            return Some(next);
        }

        None
    }

    pub fn as_joining_relocate_candidate(&self) -> JoiningRelocateCandidate {
        JoiningRelocateCandidate(self.clone())
    }

    pub fn failure_event(&self, event: Event) -> Self {
        Self {
            failure: Some(event),
            ..self.clone()
        }
    }
}