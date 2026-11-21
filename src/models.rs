//! Structs que espelham as linhas das tabelas do banco.
//! Não derivam `Serialize` de propósito: o que vai para o cliente são os DTOs de resposta.

pub mod academic_education;
pub mod certification;
pub mod experience;
pub mod profile;
pub mod skill;
pub mod social_link;
pub mod user;
