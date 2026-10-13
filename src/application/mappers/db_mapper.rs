pub trait DbMapper<Entity, DbModel> {
    fn to_db(entity: Entity) -> DbModel;
    fn to_entity(model: DbModel) -> Entity;
}
