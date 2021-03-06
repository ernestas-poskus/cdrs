#[macro_use]
extern crate cdrs;
#[macro_use]
extern crate cdrs_helpers_derive;
#[macro_use]
extern crate maplit;

use cdrs::authenticators::NoneAuthenticator;
use cdrs::cluster::{Cluster, Session};
use cdrs::query::*;
use cdrs::load_balancing::RoundRobin;
use cdrs::transport::TransportTcp;

use cdrs::types::prelude::*;
use cdrs::types::from_cdrs::FromCDRSByName;
use cdrs::frame::IntoBytes;

type CurrentSession = Session<RoundRobin<TransportTcp>, NoneAuthenticator>;

#[derive(Clone, Debug, IntoCDRSValue, TryFromRow, PartialEq)]
struct RowStruct {
  key: i32,
}

impl RowStruct {
  fn into_query_values(self) -> QueryValues {
    /// **IMPORTANT NOTE:** query values should be WITHOUT NAMES
    /// https://github.com/apache/cassandra/blob/trunk/doc/native_protocol_v4.spec#L413
    query_values!(self.key)
  }
}

fn main() {
  let cluster = Cluster::new(vec!["127.0.0.1:9042"], NoneAuthenticator {});
  let mut no_compression = cluster
    .connect(RoundRobin::new())
    .expect("No compression connection error");

  create_keyspace(&mut no_compression);
  create_table(&mut no_compression);

  let insert_struct_cql = "INSERT INTO test_ks.my_test_table (key) VALUES (?)";
  let prepared_query = no_compression
    .prepare(insert_struct_cql)
    .expect("Prepare query error");

  for k in 100..110 {
    let row = RowStruct { key: k as i32 };

    insert_row(&mut no_compression, row, &prepared_query);
  }

  batch_few_queries(&mut no_compression, &insert_struct_cql);
}

fn create_keyspace(session: &mut CurrentSession) {
  let create_ks: &'static str = "CREATE KEYSPACE IF NOT EXISTS test_ks WITH REPLICATION = { \
                                 'class' : 'SimpleStrategy', 'replication_factor' : 1 };";
  session.query(create_ks).expect("Keyspace creation error");
}

fn create_table(session: &mut CurrentSession) {
  let create_table_cql = "CREATE TABLE IF NOT EXISTS test_ks.my_test_table (key int PRIMARY KEY);";
  session
    .query(create_table_cql)
    .expect("Table creation error");
}

fn insert_row(session: &mut CurrentSession, row: RowStruct, prepared_query: &PreparedQuery) {
  session
    .exec_with_values(prepared_query, row.into_query_values())
    .expect("exec_with_values error");
}

fn batch_few_queries(session: &mut CurrentSession, query: &str) {
  let prepared_query = session.prepare(query).expect("Prepare query error");
  let row_1 = RowStruct { key: 1001 as i32 };
  let row_2 = RowStruct { key: 2001 as i32 };

  let batch = BatchQueryBuilder::new()
    .add_query_prepared(prepared_query, row_1.into_query_values())
    .add_query(query, row_2.into_query_values())
    .finalize()
    .expect("batch builder");

  session.batch_with_params(batch).expect("batch query error");
}
