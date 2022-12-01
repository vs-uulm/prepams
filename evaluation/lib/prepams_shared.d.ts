/* tslint:disable */
/* eslint-disable */
/**
*/
export class Issuer {
  free(): void;
/**
* @returns {any}
*/
  serialize(): any;
/**
* @param {any} o
* @returns {Issuer}
*/
  static deserialize(o: any): Issuer;
/**
* @returns {any}
*/
  serializeBase64(): any;
/**
* @param {string} data
* @returns {Issuer}
*/
  static deserializeBase64(data: string): Issuer;
/**
* @returns {Uint8Array}
*/
  serializeBinary(): Uint8Array;
/**
* @param {Uint8Array} data
* @returns {Issuer}
*/
  static deserializeBinary(data: Uint8Array): Issuer;
/**
*/
  constructor();
/**
* @param {any} request
* @returns {any}
*/
  issueCredential(request: any): any;
/**
* @param {Reward} reward
* @param {any} approvedKeys
* @returns {boolean}
*/
  static checkRewardSignature(reward: Reward, approvedKeys: any): boolean;
/**
* @param {string} resource
* @param {string} signature
* @param {string} publicKey
* @returns {boolean}
*/
  checkResourceSignature(resource: string, signature: string, publicKey: string): boolean;
/**
* @param {any} request
* @param {any} transactions
* @param {any} spend
* @returns {any}
*/
  checkPayoutRequest(request: any, transactions: any, spend: any): any;
/**
* @returns {any}
*/
  readonly publicKey: any;
}
/**
*/
export class Organizer {
  free(): void;
/**
* @returns {any}
*/
  serialize(): any;
/**
* @param {any} o
* @returns {Organizer}
*/
  static deserialize(o: any): Organizer;
/**
* @returns {any}
*/
  serializeBase64(): any;
/**
* @param {string} data
* @returns {Organizer}
*/
  static deserializeBase64(data: string): Organizer;
/**
* @returns {Uint8Array}
*/
  serializeBinary(): Uint8Array;
/**
* @param {Uint8Array} data
* @returns {Organizer}
*/
  static deserializeBinary(data: Uint8Array): Organizer;
/**
* @param {string} identity
* @param {any} issuerPublicKey
* @param {Uint8Array} seed
*/
  constructor(identity: string, issuerPublicKey: any, seed: Uint8Array);
/**
* @param {Participation} participation
* @returns {boolean}
*/
  checkParticipation(participation: Participation): boolean;
/**
* @param {string} resource
* @returns {string}
*/
  signResource(resource: string): string;
/**
* @param {Participation} participation
* @returns {Reward}
*/
  issueReward(participation: Participation): Reward;
/**
* @returns {any}
*/
  readonly id: any;
/**
* @returns {any}
*/
  readonly identity: any;
/**
* @returns {any}
*/
  readonly publicKey: any;
/**
* @returns {any}
*/
  readonly role: any;
}
/**
*/
export class Participant {
  free(): void;
/**
* @returns {any}
*/
  serialize(): any;
/**
* @param {any} o
* @returns {Participant}
*/
  static deserialize(o: any): Participant;
/**
* @returns {any}
*/
  serializeBase64(): any;
/**
* @param {string} data
* @returns {Participant}
*/
  static deserializeBase64(data: string): Participant;
/**
* @returns {Uint8Array}
*/
  serializeBinary(): Uint8Array;
/**
* @param {Uint8Array} data
* @returns {Participant}
*/
  static deserializeBinary(data: Uint8Array): Participant;
/**
* @param {string} identity
*/
  constructor(identity: string);
/**
* @returns {any}
*/
  data(): any;
/**
* @param {any} issuerPublicKey
* @param {Uint8Array} seed
* @returns {any}
*/
  requestCredential(issuerPublicKey: any, seed: Uint8Array): any;
/**
* @param {any} issueResponse
*/
  retrieveCredential(issueResponse: any): void;
/**
* @param {Resource} resource
* @returns {Participation}
*/
  participate(resource: Resource): Participation;
/**
* @param {any} transactions
* @param {any} spend
* @returns {any}
*/
  getBalance(transactions: any, spend: any): any;
/**
* @param {number} value
* @param {string} target
* @param {string} recipient
* @param {any} transactions
* @param {any} spend
* @returns {any}
*/
  requestPayout(value: number, target: string, recipient: string, transactions: any, spend: any): any;
/**
* @returns {any}
*/
  readonly id: any;
/**
* @returns {any}
*/
  readonly identity: any;
/**
* @returns {any}
*/
  readonly role: any;
}
/**
*/
export class Participation {
  free(): void;
/**
* @returns {any}
*/
  serialize(): any;
/**
* @param {any} o
* @returns {Participation}
*/
  static deserialize(o: any): Participation;
/**
* @returns {any}
*/
  serializeBase64(): any;
/**
* @param {string} data
* @returns {Participation}
*/
  static deserializeBase64(data: string): Participation;
/**
* @returns {Uint8Array}
*/
  serializeBinary(): Uint8Array;
/**
* @param {Uint8Array} data
* @returns {Participation}
*/
  static deserializeBinary(data: Uint8Array): Participation;
}
/**
*/
export class Resource {
  free(): void;
/**
* @param {number} reward
*/
  constructor(reward: number);
/**
* @returns {any}
*/
  serialize(): any;
/**
* @param {any} o
* @returns {Resource}
*/
  static deserialize(o: any): Resource;
/**
* @returns {any}
*/
  serializeBase64(): any;
/**
* @param {string} data
* @returns {Resource}
*/
  static deserializeBase64(data: string): Resource;
/**
* @returns {Uint8Array}
*/
  serializeBinary(): Uint8Array;
/**
* @param {Uint8Array} data
* @returns {Resource}
*/
  static deserializeBinary(data: Uint8Array): Resource;
/**
* @returns {any}
*/
  readonly id: any;
}
/**
*/
export class Reward {
  free(): void;
/**
* @returns {any}
*/
  serialize(): any;
/**
* @param {any} o
* @returns {Reward}
*/
  static deserialize(o: any): Reward;
/**
* @returns {any}
*/
  serializeBase64(): any;
/**
* @param {string} data
* @returns {Reward}
*/
  static deserializeBase64(data: string): Reward;
/**
* @returns {Uint8Array}
*/
  serializeBinary(): Uint8Array;
/**
* @param {Uint8Array} data
* @returns {Reward}
*/
  static deserializeBinary(data: Uint8Array): Reward;
}
