create table profile (
    "id" bigserial primary key,
    "chain_id" varchar(500) NOT NULL,
    "created_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "user_name" varchar(50) NOT NULL,
    "full_name" varchar(100) NOT NULL,
    "description" varchar(250) NOT NULL,
    "main_url" varchar(250),
    "avatar" bytea
);

create table follow (
    "id" bigserial primary key,
    "created_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "follower_id" bigserial NOT NULL,
    "following_id" bigserial NOT NULL,

    constraint fk_profile_follower foreign key(follower_id) references profile(id),
    constraint fk_profile_following foreign key(following_id) references profile(id)
);

create table post (
    "id" bigserial primary key,
    "chain_id" varchar(500) NOT NULL,
    "created_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "user_id" bigserial NOT NULL,
    "message"  varchar(140),
    "image" bytea,

    constraint fk_profile foreign key(user_id) references profile(id)
);

create table post_response (
    "id" bigserial primary key,
    "created_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "respondee_post_id" bigserial NOT NULL,
    "responder_post_id" bigserial NOT NULL,

    constraint fk_respondee_post foreign key(respondee_post_id) references post(id),
    constraint fk_responder_post foreign key(responder_post_id) references post(id)
);

create table post_share (
    "id" bigserial primary key,
    "created_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" timestamptz(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "sharee_post_id" bigserial NOT NULL,
    "sharer_post_id" bigserial NOT NULL,

    constraint fk_sharee_post foreign key(sharee_post_id) references post(id),
    constraint fk_sharer_post foreign key(sharer_post_id) references post(id)
);