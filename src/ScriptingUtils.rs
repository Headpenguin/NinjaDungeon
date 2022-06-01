use fs;
use rlua::Lua;

pub struct LuaScript(Vec<(u8)>);

impl LuaScript {
    pub fn new(state: &Lua, filename: &str) -> Result<LuaScript> {
        Ok(LuaScript(state.context(|context| {
            context.load(fs::read_to_string(filename)?).into_function()?.dump()?
        })))
    }
    pub unsafe fn fromPrecompiled(filename: &str) -> Result<LuaScript> {
        Ok(LuaScript(fs::read(filename)?))
    }
}

