start by running in docker compose whilst standing in the directory of the file

docker compose up -d --build

when that done building exec in one of the configservers:

docker exec -it cfgsvr1 mongosh

then initiate the config server:

rs.initiate({
  _id: "cfgrs",
  configsvr: true,
  members: [
    { _id: 0, host: "cfgsvr1:27017" },
    { _id: 1, host: "cfgsvr2:27017" },
    { _id: 2, host: "cfgsvr3:27017" }
  ]
})

when that is done make sure there is a primary and two secondary servers (you might need to exec out and in again or esc then enter)

rs.status()

then exec in one of shard1 servers:

docker exec -it shard1svr1 mongosh

then initiate it:

rs.initiate({
  _id: "shard1rs",
  members: [
    { _id: 0, host: "shard1svr1:27017" },
    { _id: 1, host: "shard1svr2:27017" },
    { _id: 2, host: "shard1svr3:27017" }
  ]
})

when that is done make sure there is a primary and two secondary server (you might need to exec out and in again or esc then enter)

rs.status()

then do it again for shard2 exec in one of shard2 servers:

docker exec -it shard2svr1 mongosh

then initiate it:

rs.initiate({
  _id: "shard2rs",
  members: [
    { _id: 0, host: "shard2svr1:27017" },
    { _id: 1, host: "shard2svr2:27017" },
    { _id: 2, host: "shard2svr3:27017" }
  ]
})

when that is done make sure there is a primary and two secondary server (you might need to exec out and in again or esc then enter)

rs.status()

Lastly, to start the mongos router, run:

docker exec -it mongos mongosh

then we need to add the shards

sh.addShard("shard1rs/shard1svr1:27017,shard1svr2:27017,shard1svr3:27017")

make sure it correct under shards (you might need to exec out and in again or esc then enter):

sh.status()

then do it again with shard2 

sh.addShard("shard2rs/shard2svr1:27017,shard2svr2:27017,shard2svr3:27017")

make sure it correct under shards (you might need to exec out and in again or esc then enter):

sh.status()

enjoy sharding.


 
