use error;
use query::{QueryParams, QueryParamsBuilder, QueryValues};
use frame::{Flag, Frame, IntoBytes};
use frame::parser::parse_frame;
use types::CBytesShort;
use cluster::{GetCompressor, GetTransport};
use transport::CDRSTransport;

pub type PreparedQuery = CBytesShort;

pub trait ExecExecutor<'a, T: CDRSTransport + 'a>
  : GetTransport<'a, T> + GetCompressor<'a> {
  fn exec_with_params_tw(&mut self,
                         prepared: &PreparedQuery,
                         query_parameters: QueryParams,
                         with_tracing: bool,
                         with_warnings: bool)
                         -> error::Result<Frame> {
    let mut flags = vec![];
    if with_tracing {
      flags.push(Flag::Tracing);
    }
    if with_warnings {
      flags.push(Flag::Warning);
    }

    let options_frame = Frame::new_req_execute(prepared, query_parameters, flags).into_cbytes();
    let ref compression = self.get_compressor();
    let transport = self.get_transport().ok_or("Unable to get transport")?;

    (transport.write(options_frame.as_slice()))?;
    parse_frame(transport, compression)
  }

  fn exec_with_params(&mut self,
                      prepared: &PreparedQuery,
                      query_parameters: QueryParams)
                      -> error::Result<Frame> {
    self.exec_with_params_tw(prepared, query_parameters, false, false)
  }

  fn exec_with_values_tw<V: Into<QueryValues>>(&mut self,
                                               prepared: &PreparedQuery,
                                               values: V,
                                               with_tracing: bool,
                                               with_warnings: bool)
                                               -> error::Result<Frame> {
    let query_params_builder = QueryParamsBuilder::new();
    let query_params = query_params_builder.values(values.into()).finalize();
    self.exec_with_params_tw(prepared, query_params, with_tracing, with_warnings)
  }

  fn exec_with_values<V: Into<QueryValues>>(&mut self,
                                            prepared: &PreparedQuery,
                                            values: V)
                                            -> error::Result<Frame> {
    self.exec_with_values_tw(prepared, values, false, false)
  }

  fn exec_tw(&mut self,
             prepared: &PreparedQuery,
             with_tracing: bool,
             with_warnings: bool)
             -> error::Result<Frame> {
    let query_params = QueryParamsBuilder::new().finalize();
    self.exec_with_params_tw(prepared, query_params, with_tracing, with_warnings)
  }

  fn exec(&'a mut self, prepared: &PreparedQuery) -> error::Result<Frame> {
    self.exec_tw(prepared, false, false)
  }
}
