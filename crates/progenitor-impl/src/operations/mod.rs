// Copyright 2026 Oxide Computer Company

//! Spec-to-operation lowering and response-set analysis shared by every
//! code-generation backend (client emitters, CLI, httpmock, server).

pub(crate) mod lower;
pub(crate) mod responses;
