DEFINE TABLE OVERWRITE activity TYPE NORMAL SCHEMAFULL 
    PERMISSIONS 
        FOR select FULL
        FOR create, update FULL
        FOR delete NONE;

DEFINE FIELD OVERWRITE content ON activity TYPE string ASSERT $value != NONE;
DEFINE FIELD OVERWRITE tags ON activity TYPE array<string>;
DEFINE FIELD OVERWRITE source ON activity TYPE option<string>;
DEFINE FIELD OVERWRITE created_at ON activity TYPE datetime DEFAULT time::now();
