create table todos (
  id bigint auto_increment not null,
  text varchar(255) not null,
  done tinyint(1) not null,
  created_at datetime not null,
  updated_at datetime not null,

  primary key (id)
);
