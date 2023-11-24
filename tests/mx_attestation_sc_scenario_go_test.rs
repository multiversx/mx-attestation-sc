use multiversx_sc_scenario::*;

fn world() -> ScenarioWorld {
	ScenarioWorld::vm_go()
}

#[test]
fn main_go() {
	world().run("scenarios/main.scen.json");
}
