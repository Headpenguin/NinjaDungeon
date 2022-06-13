use std::fs;
use std::path::Path;
use std::io;
use rlua::{Lua, Function, Context, FromLuaMulti, ToLuaMulti};

pub struct LuaFunction(Vec<u8>);

impl LuaFunction {
    pub fn fromFile(state: &Lua, filename: &str) -> rlua::Result<LuaFunction> {
		Ok(LuaFunction(state.context(|context| {
            Ok(context.load(&fs::read_to_string(filename).map_err(Self::convertError)?).into_function()?.dump()?)
        })?))
    }
	pub fn fromFunctionNames<'a, N, P, F>(state: &Lua, filenames: F, names: N) -> rlua::Result<Vec<rlua::Result<LuaFunction>>> where
	P: AsRef<Path>,
	F: Iterator<Item=P>,
	N: Iterator<Item=&'a str>, {
		state.context(|context| {
			for filename in filenames {
				context.load(&fs::read_to_string(filename).map_err(Self::convertError)?).exec()?;
			}
			let globals = context.globals();
			let mut v = vec![];
			for function in names {
				v.push(match globals.get::<_, Function>(function) {
					Ok(function) => function.dump().map(|function| LuaFunction(function)),
					Err(e) => Err(e),
				});
			}
			Ok(v)
		})
	}
    pub unsafe fn fromPrecompiled(filename: &str) -> io::Result<LuaFunction> {
        Ok(LuaFunction(fs::read(filename)?))
    }
	pub fn callFromContext<'lua, A, R>(&self, context: Context<'lua>, args: A) -> rlua::Result<R> where
	A: ToLuaMulti<'lua>,
	R: FromLuaMulti<'lua>, {
		unsafe{context.load(&self.0).into_function_allow_binary()?}.call(args)
	}
    pub fn call<'lua, A, R>(&self, state: &Lua, args: A) -> rlua::Result<R> where
    A: ToLuaMulti<'lua>,
    R: FromLuaMulti<'lua> {
        state.context(|c| self.callFromContext(c, args))
    }
	fn convertError(error: io::Error) -> rlua::Error {
		rlua::Error::external(error)
	}
}

