import type { Principal } from '@dfinity/principal';
export type CancelOrderReceipt = { 'Ok' : OrderId } |
  { 'Err' : { 'NotAllowed' : null } | { 'NotExistingOrder' : null } };
export type DepositReceipt = { 'Ok' : bigint } |
  { 'Err' : { 'TransferFailure' : null } | { 'BalanceLow' : null } };
export interface Order {
  'id' : OrderId,
  'to' : Token,
  'dip_symbol' : string,
  'fromAmount' : bigint,
  'submitted' : Time,
  'owner' : Principal,
  'from' : Token,
  'price' : number,
  'toAmount' : bigint,
}
export type OrderId = number;
export type OrderPlacementReceipt = { 'Ok' : Order } |
  { 'Err' : { 'InvalidOrder' : null } | { 'OrderBookFull' : null } };
export type Time = bigint;
export type Token = Principal;
export type WithdrawReceipt = { 'Ok' : bigint } |
  { 'Err' : { 'TransferFailure' : null } | { 'BalanceLow' : null } };
export interface _SERVICE {
  'balance' : (arg_0: Token) => Promise<bigint>,
  'cancel_order' : (arg_0: OrderId) => Promise<CancelOrderReceipt>,
  'check_order' : (arg_0: OrderId) => Promise<[] | [Order]>,
  'deposit' : (arg_0: Token) => Promise<DepositReceipt>,
  'deposit_address' : () => Promise<Array<number>>,
  'list_order' : () => Promise<Array<Order>>,
  'place_order' : (
      arg_0: Token,
      arg_1: bigint,
      arg_2: Token,
      arg_3: bigint,
    ) => Promise<OrderPlacementReceipt>,
  'symbol' : (arg_0: Token) => Promise<string>,
  'whoami' : () => Promise<Principal>,
  'withdraw' : (arg_0: Token, arg_1: bigint) => Promise<WithdrawReceipt>,
}
