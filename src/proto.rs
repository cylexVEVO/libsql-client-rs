//! `proto` contains libSQL/sqld/hrana wire protocol.

#[cfg(feature = "hrana_backend")]
pub use hrana_client::proto::{
    Batch, BatchReq, BatchResp, BatchResult, ClientMsg, Col, Error, ExecuteReq, ExecuteResp,
    OpenStreamReq, Request, Response, ServerMsg, Stmt, StmtResult, Value,
};
#[cfg(not(feature = "hrana_backend"))]
pub use hrana_client_proto::{
    Batch, BatchReq, BatchResp, BatchResult, ClientMsg, Col, Error, ExecuteReq, ExecuteResp,
    OpenStreamReq, Request, Response, ServerMsg, Stmt, StmtResult, Value,
};
