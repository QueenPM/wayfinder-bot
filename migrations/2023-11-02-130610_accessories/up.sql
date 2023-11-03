-- Your SQL goes here
create table accessories (
    id serial primary key,
    name varchar(255) not null,
    description text,
    icon_url varchar(255),
    icon_emoji varchar(255),
    tier int not null,
    url varchar(255) not null,
)