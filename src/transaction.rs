use crate::*;
use tvm::types::UInt256;
use futures::stream::Stream;
use ton_block::{TransactionProcesingStatus, MessageId};

pub type TransactionId = UInt256;

#[derive(Debug)]
pub struct Transaction {
    tr: ton_block::Transaction,
}

const TR_TABLE_NAME: &str = "transactions";

// The struct represents performed transaction and allows to access their properties.
#[allow(dead_code)]
impl Transaction {

    // Asynchronously loads a Transaction instance or None if transaction with given id is not exists
    pub fn load(id: TransactionId) -> SdkResult<Box<Stream<Item = Option<Transaction>, Error = SdkError>>> {
        let map = db_helper::load_record(TR_TABLE_NAME, &id.to_hex_string())?
            .and_then(|val| {
                if val == serde_json::Value::Null {
                    Ok(None)
                } else {
                    let tr: ton_block::Transaction = serde_json::from_value(val)
                        .map_err(|err| SdkErrorKind::InvalidData(format!("error parsing transaction: {}", err)))?;

                    Ok(Some(Transaction { tr }))
                }
            });

        Ok(Box::new(map))
    }

    // Returns transaction's processing status
    pub fn status(&self) -> TransactionProcesingStatus {
        self.tr.processing_status()
    }

    // Returns id of transaction's input message if it exists
    pub fn in_message_id(&self) -> Option<MessageId> {
        self.tr.in_message().map(|m| m.client_ref_unwrap().clone())
    }

    // Asynchronously loads an instance of transaction's input message
    pub fn load_in_message(&self) -> SdkResult<Box<Stream<Item = Option<Message>, Error = SdkError>>> {
        match self.in_message_id() {
            Some(m) => Message::load(m),
            None => bail!(SdkErrorKind::InvalidOperation("transaction doesn't have inbound message".into()))
        }
    }

    // Returns id of transaction's out messages if it exists
    pub fn out_messages_id(&self) -> &Vec<MessageId> {
        &self.tr.out_msgs_sdk()
    }

    // Returns message's identifier
    pub fn id(&self) -> TransactionId {
        self.tr.id.clone()
    }

    // Asynchronously loads an instances of transaction's out messages
    pub fn load_out_messages(&self) -> SdkResult<Box<Stream<Item = Option<Message>, Error = SdkError>>> {
        let mut msg_id_iter = self.out_messages_id().iter();
        if let Some(id) = msg_id_iter.next().clone() {
            let mut stream = Message::load(id.clone())?;
            for id in msg_id_iter {
                stream = Box::new(stream.chain(Message::load(id.clone())?));
            }
            Ok(stream)
        } else {
            // TODO how to return empty Stream?
            bail!(SdkErrorKind::NoData);
        }
    }
}