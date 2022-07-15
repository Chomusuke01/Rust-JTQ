 
 create table visitor{
    id serial primary key,
    name varchar(255) not null,
    username varchar(255) not null,
    password varchar(255) not null,
    phone_number varchar(255) not null,
    accepted_terms boolean not null default false,
    accepted_comercial boolean not null default false,
    user_type boolean not null default false
 };