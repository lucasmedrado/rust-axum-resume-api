//! Acesso ao banco. Cada função executa uma única query e devolve modelos ou `Option`;
//! decidir o que "não encontrado" significa é responsabilidade da camada de serviço.

pub mod academic_education_repository;
pub mod certification_repository;
pub mod experience_repository;
pub mod profile_repository;
pub mod skill_repository;
pub mod social_link_repository;
pub mod user_repository;
