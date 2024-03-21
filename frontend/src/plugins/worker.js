import { init, Participant, Resource, Participation } from 'prepams-shared';
init();

self.addEventListener('message', ({ data }) => {
  try {
    const args = data.args;
    let result = null;

    switch (args.call) {
      case 'participate': {
        const resource = new Resource(
          args.resource.id,
          args.resource.name || '',
          args.resource.abstract || '',
          args.resource.description || '',
          args.resource.duration || '',
          args.resource.reward,
          args.resource.webBased && true || false,
          args.resource.studyUrl || '',
          args.resource.qualifier,
          args.resource.disqualifier,
          args.resource.constraints 
        );

        const credential = Participant.deserialize(new Uint8Array(args.credential));
        const participation = credential.participate(resource);
        result = participation;
        break;
      }

      case 'verify': {
        const p = Participation.deserialize(new Uint8Array(args.participation));

        result = {
          study: p.id,
          id: args.id,
          reward: p.reward,
          valid: p.verify(),
          data: args.participation,
          rewarded: args.rewarded
        };
        break;
      }

      case 'payout': {
        const credential = Participant.deserialize(new Uint8Array(args.credential));
        const request = credential.requestPayout(
          args.amount,
          args.target,
          args.recipient,
          args.nulls,
          args.ledger
        );

        result = {
          costs: request.costs,
          proof: request.proof
        };
        break;
      }

      default:
        throw new Error(`illegal call: ${args.call}`);
    }

    self.postMessage({
      id: data.id,
      error: false,
      result: result
    });
  } catch (e) {
    self.postMessage({
      id: data.id,
      error: true,
      result: e
    });
  }
});
