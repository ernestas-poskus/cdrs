use error;
use frame::{Flag, Frame, IntoBytes};
use query::{Query, QueryParams, QueryParamsBuilder, QueryValues};
use cluster::{GetCompressor, GetTransport};
use transport::CDRSTransport;
use frame::parser::parse_frame;

pub trait QueryExecutor<'a, T: CDRSTransport + 'a>
  : GetTransport<'a, T> + GetCompressor<'a> {
  fn query_with_params_tw<Q: ToString>(&mut self,
                                       query: Q,
                                       query_params: QueryParams,
                                       with_tracing: bool,
                                       with_warnings: bool)
                                       -> error::Result<Frame> {
    let query = Query { query: query.to_string(),
                        params: query_params, };

    let mut flags = vec![];

    if with_tracing {
      flags.push(Flag::Tracing);
    }

    if with_warnings {
      flags.push(Flag::Warning);
    }

    let query_frame = Frame::new_query(query, flags).into_cbytes();
    let ref compression = self.get_compressor();
    let transport = self.get_transport().ok_or("Unable to get transport")?;
    try!(transport.write(query_frame.as_slice()));
    parse_frame(transport, compression)
  }

  /// Executes a query with default parameters:
  /// * TDB
  fn query<Q: ToString>(&mut self, query: Q) -> error::Result<Frame> {
    self.query_tw(query, false, false)
  }

  /// Executes a query with ability to trace it and see warnings, and default parameters:
  /// * TBD
  fn query_tw<Q: ToString>(&mut self,
                           query: Q,
                           with_tracing: bool,
                           with_warnings: bool)
                           -> error::Result<Frame> {
    let query_params = QueryParamsBuilder::new().finalize();
    self.query_with_params_tw(query, query_params, with_tracing, with_warnings)
  }

  /// Executes a query with bounded values (either with or without names).
  fn query_with_values<Q: ToString, V: Into<QueryValues>>(&mut self,
                                                          query: Q,
                                                          values: V)
                                                          -> error::Result<Frame> {
    self.query_with_values_tw(query, values, false, false)
  }

  /// Executes a query with bounded values (either with or without names)
  /// and ability to see warnings, trace a request and default parameters.
  fn query_with_values_tw<Q: ToString, V: Into<QueryValues>>(&mut self,
                                                             query: Q,
                                                             values: V,
                                                             with_tracing: bool,
                                                             with_warnings: bool)
                                                             -> error::Result<Frame> {
    let query_params_builder = QueryParamsBuilder::new();
    let query_params = query_params_builder.values(values.into()).finalize();
    self.query_with_params_tw(query, query_params, with_tracing, with_warnings)
  }

  /// Executes a query with query params without warnings and tracing.
  fn query_with_params<Q: ToString>(&mut self,
                                    query: Q,
                                    query_params: QueryParams)
                                    -> error::Result<Frame> {
    self.query_with_params_tw(query, query_params, false, false)
  }
}
