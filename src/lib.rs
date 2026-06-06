use diesel::{
    associations::HasTable,
    query_builder::{AsQuery, DeleteStatement, InsertStatement, IntoUpdateTarget, UpdateStatement},
    query_dsl::methods::FindDsl,
    AsChangeset, Insertable,
};
use diesel_async::{
    methods::{ExecuteDsl, LoadQuery},
    AsyncConnectionCore, RunQueryDsl,
};

#[cfg(feature = "derive")]
pub use diesel_crudable_derive::{Creatable, Deletable, Model, Readable, Updatable};

type Delete<T> = DeleteStatement<<T as HasTable>::Table, <T as IntoUpdateTarget>::WhereClause>;

type Find<T> = diesel::dsl::Find<<T as HasTable>::Table, <T as Model>::IdType>;

type InsertValues<F, T> = <F as Insertable<<T as HasTable>::Table>>::Values;

type Insert<F, T> = InsertStatement<<T as HasTable>::Table, InsertValues<F, T>>;

type Update<F, T> = UpdateStatement<
    <T as HasTable>::Table,
    <Find<T> as IntoUpdateTarget>::WhereClause,
    <F as AsChangeset>::Changeset,
>;

pub trait Crud: Creatable + Readable + Updatable + Deletable {}
impl<T: Creatable + Readable + Updatable + Deletable> Crud for T {}

pub trait Model: HasTable + Sized + Send + 'static {
    type IdType: Send;
}

pub trait Creatable: HasTable + Sized + Send + 'static {
    fn create<'a, 'conn, F, Conn>(
        conn: &'conn mut Conn,
        form: F,
    ) -> impl Future<Output = diesel::result::QueryResult<Self>> + Send
    where
        Conn: AsyncConnectionCore,
        F: Insertable<Self::Table> + Send + 'a,
        Insert<F, Self>: LoadQuery<'a, Conn, Self>,
    {
        #[cfg(not(debug_assertions))]
        {
            Self::create_query(form).get_result::<Self>(conn)
        }

        #[cfg(debug_assertions)]
        async move {
            let rows = Self::create_query(form).get_results(conn).await?;
            debug_assert!(
                rows.len() == 1,
                "create() inserted {} rows - you passed a batch; use create_many()",
                rows.len()
            );
            rows.into_iter()
                .next()
                .ok_or(diesel::result::Error::NotFound)
        }
    }

    fn create_many<'a, 'conn, F, Conn>(
        conn: &'conn mut Conn,
        form: F,
    ) -> impl Future<Output = diesel::result::QueryResult<Vec<Self>>> + Send
    where
        Conn: AsyncConnectionCore,
        F: Insertable<Self::Table> + Send + 'a,
        Insert<F, Self>: LoadQuery<'a, Conn, Self>,
    {
        Self::create_query(form).get_results::<Self>(conn)
    }

    fn create_query<F>(form: F) -> Insert<F, Self>
    where
        F: Insertable<Self::Table>,
    {
        diesel::insert_into(Self::table()).values(form)
    }
}

pub trait Updatable: Model {
    fn update<'a, 'conn, F, Conn>(
        conn: &'conn mut Conn,
        id: Self::IdType,
        form: F,
    ) -> impl Future<Output = diesel::result::QueryResult<Self>> + Send
    where
        Conn: AsyncConnectionCore,
        Self::Table: FindDsl<Self::IdType>,
        Find<Self>: IntoUpdateTarget + HasTable<Table = Self::Table>,
        F: AsChangeset<Target = Self::Table> + Send + 'a,
        Update<F, Self>: AsQuery + LoadQuery<'a, Conn, Self>,
    {
        Self::update_query(id, form).get_result::<Self>(conn)
    }
    fn update_query<F>(id: Self::IdType, form: F) -> Update<F, Self>
    where
        Self::Table: FindDsl<Self::IdType>,
        Find<Self>: IntoUpdateTarget + HasTable<Table = Self::Table>,
        F: AsChangeset<Target = Self::Table>,
        Update<F, Self>: AsQuery,
    {
        diesel::update(Self::table().find(id)).set(form)
    }
}

pub trait Readable: Model {
    fn read<'conn, Conn>(
        conn: &'conn mut Conn,
        id: Self::IdType,
    ) -> impl Future<Output = diesel::result::QueryResult<Self>> + Send
    where
        Conn: AsyncConnectionCore,
        Self::Table: FindDsl<Self::IdType>,
        Find<Self>: LoadQuery<'static, Conn, Self>,
    {
        Self::read_query(id).get_result::<Self>(conn)
    }

    fn all<'conn, Conn>(
        conn: &'conn mut Conn,
    ) -> impl Future<Output = diesel::result::QueryResult<Vec<Self>>> + Send
    where
        Conn: AsyncConnectionCore,
        Self::Table: LoadQuery<'static, Conn, Self>,
    {
        Self::table().load::<Self>(conn)
    }

    fn read_query(id: Self::IdType) -> Find<Self>
    where
        Self::Table: FindDsl<Self::IdType>,
    {
        Self::table().find(id)
    }
}

pub trait Deletable: Model {
    fn delete<'conn, Conn>(
        conn: &'conn mut Conn,
        id: Self::IdType,
    ) -> impl Future<Output = diesel::result::QueryResult<usize>> + Send
    where
        Conn: AsyncConnectionCore,
        Self::Table: FindDsl<Self::IdType>,
        Find<Self>: IntoUpdateTarget,
        Delete<Find<Self>>: ExecuteDsl<Conn>,
    {
        diesel::delete(Self::table().find(id)).execute(conn)
    }

    fn delete_query(id: Self::IdType) -> Delete<Find<Self>>
    where
        Self::Table: FindDsl<Self::IdType>,
        Find<Self>: IntoUpdateTarget,
    {
        diesel::delete(Self::table().find(id))
    }
}
