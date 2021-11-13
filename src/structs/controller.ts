import { UserData } from './UserData';
import { handleRoles } from './roleHandling';
import { CONSTANTS } from '../../config';
import { createHmac } from 'crypto';
import type { FastifyReply } from 'fastify';
import type { APIRequest, IBody } from '../../server';

export const list_userdata = function (req: APIRequest, res: FastifyReply): void {
  UserData.find({}, function (err, entry) {
    if (err) return res.status(400).send({ error: err.message });
    res.status(200).send({ data: entry });
  });
};

export const write_userdata = function (req: APIRequest, res: FastifyReply): void {
  const new_entry = new UserData(req.body);
  new_entry.save(function (err, entry) {
    if (err) return res.status(400).send({ error: err.message });
    res.status(200).send({ data: entry });
  });
};

export const read_userdata = function (req: APIRequest, res: FastifyReply): void {
  UserData.findOne(
    { playerId: req.query.playerId, playerToken: req.body.playerToken },
    null,
    {},
    function (err, entry) {
      if (err) return res.status(400).send({ error: err.message });
      res.status(200).send({ data: entry });
    },
  );
};

export const update_userdata = function (req: APIRequest, res: FastifyReply): void {
  const token = createHmac('sha1', process.env.USERDATA_AUTH)
    .update(req.query.playerId)
    .update(req.body.playerToken)
    .digest('hex');
  const updatableEntry: Omit<IBody, 'playerToken'> = {
    betaTester: req.body.betaTester ?? undefined,
    dino_rank: req.body.dino_rank ?? undefined,
    metabits: req.body.metabits ?? undefined,
    prestige_rank: req.body.prestige_rank ?? undefined,
    singularity_speedrun_time: req.body.singularity_speedrun_time ?? undefined,
    all_sharks_obtained: req.body.all_sharks_obtained ?? undefined,
    all_hidden_achievements_obtained: req.body.all_hidden_achievements_obtained ?? undefined,
  };
  UserData.findOneAndUpdate(
    { token },
    {
      $set: {
        ...updatableEntry,
        edited_timestamp: Date.now(),
      },
    },
    { new: true },
    async function (err, entry) {
      if (err) return res.status(400).send({ error: err.message });
      if (entry) {
        try {
          const member = await req.client.guilds.cache
            .get(CONSTANTS.C2SGUILD)
            .members.fetch({ user: entry.discordId, force: true, cache: false });
          const gainedRoles = await handleRoles(req.body, member);
          if (gainedRoles.length > 0) {
            console.log(`Successfully given the following roles to ${member.user.tag}: ${gainedRoles.join(', ')}`);

            member.user
              .send(
                `You have successfully received the following roles: ${gainedRoles.join(
                  ', ',
                )}\n **congrats on your accomplishment! :tada:**`,
              )
              .catch(() =>
                console.error(
                  `${
                    member.user.tag
                  } has their DMs closed and wasn't able to be notified about the discord link giving them their role(s): ${gainedRoles.join(
                    ', ',
                  )}`,
                ),
              );
          } else
            console.log(`${member.user.tag} has reached the requirements for all of the roles but already has them.`);
        } catch (e) {
          console.error(`Error with Userdata role handler:\n ${e.message}`);
          res.status(400).send({ error: e.message });
        }
      }
      res.status(200).send({ data: entry });
    },
  );
};

export const delete_userdata = function (req: APIRequest, res: FastifyReply): void {
  UserData.findOneAndDelete(
    {
      playerId: req.query.playerId,
      playerToken: req.body.playerToken,
    },
    {},
    function (err, entry) {
      if (err) return res.status(400).send(err);
      res.status(200).send({ message: 'User Data successfully deleted', discordId: entry.discordId });
    },
  );
};
