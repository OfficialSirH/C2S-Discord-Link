import { CONSTANTS } from './config';
import fastify from 'fastify';
import type { FastifyReply, FastifyRequest } from 'fastify';
import * as dotenv from 'dotenv';
import { connect } from 'mongoose';
import * as API from './api';
import { Client } from 'discord.js';
import type { IncomingMessage, Server } from 'http';
const { C2SGUILD } = CONSTANTS;

dotenv.config();
const server = fastify(),
  client = new Client({
    intents: [],
  });

export interface IQuerystring {
  playerId: string;
}

export interface IHeaders {
  Authorization: string;
}

export interface IBody {
  playerToken: string;
  betaTester: boolean;
  metabits: number;
  dino_rank: number;
  prestige_rank: number;
  singularity_speedrun_time: number;
  all_sharks_obtained: boolean;
  all_hidden_achievements_obtained: boolean;
}

export interface APIRequest
  extends FastifyRequest<
    {
      Querystring: IQuerystring;
      Headers: IHeaders;
      Body: IBody;
    },
    Server,
    IncomingMessage
  > {
  client: Client;
}

server.route<{
  Querystring: IQuerystring;
  Headers: IHeaders;
  Body: IBody;
}>({
  method: 'POST',
  url: '/userdata',
  preHandler: (req, res) => middleware(req, res),
  handler: (request, reply) => {
    const req = { ...request, client } as APIRequest;
    API.update_userdata(req, reply);
  },
});

server.get('/', (_req, res) => {
  res.send({
    success: true,
  });
});

function middleware(req: Omit<APIRequest, 'client'>, rep: FastifyReply) {
  const { playerId } = req.query;
  const { Authorization, authorization } = req.headers;

  if (typeof req.body != 'object') return rep.status(422).send({ error: 'Malformed request' });

  const {
    playerToken,
    betaTester,
    metabits,
    dino_rank,
    prestige_rank,
    singularity_speedrun_time,
    all_sharks_obtained,
    all_hidden_achievements_obtained,
  } = req.body;
  if (!playerId)
    return rep.status(403).send({
      error: 'Missing playerId',
    });

  if (Authorization != process.env.USERDATA_AUTH && authorization != process.env.USERDATA_AUTH)
    return rep.status(403).send({ error: 'Invalid Authorization header' });
  if (!playerToken) return rep.status(403).send({ error: 'Missing playerToken' });

  if (
    betaTester == null &&
    !metabits &&
    !dino_rank &&
    !prestige_rank &&
    !singularity_speedrun_time &&
    all_sharks_obtained == null &&
    all_hidden_achievements_obtained == null
  ) {
    return rep.status(400).send({
      error:
        'The following are required: betaTester, metabits, dino_rank, prestige_rank, singularity_speedrun_time, all_sharks_obtained, all_hidden_achievements_obtained',
    });
  }
}

(async () => {
  await connect(process.env.MONGODB_KEY);
  return client.login();
})();

const PORT = process.env.PORT ?? 3000;

client.on('ready', async () => {
  try {
    const guild = client.guilds.cache.get(C2SGUILD);
    await guild.roles.fetch();
    let address: string;
    if (process.env.NODE_ENV === 'development') address = await server.listen(PORT, '0.0.0.0');
    else address = await server.listen(PORT);
    console.log('UserData RESTful API server started on: ' + address);
    client.guilds.cache.sweep(g => g.id != C2SGUILD);
  } catch (err) {
    console.error(err);
    process.exit(1);
  }
});
