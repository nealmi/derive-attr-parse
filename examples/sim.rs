use std::collections::HashMap;
use derive_attr_parse::{Fsm, Simuples};

fn main() {}

#[derive(Simuples)]
#[sim(name = "Peoples", method = "agents")]
#[allow(dead_code)]
struct People {
    items: HashMap<u64, Person>,
}

#[derive(Simuples)]
#[sim(
name = "Person",
method = "agent",
input_name = "PersonInput",
output_name = "PersonOutput"
)]
#[allow(dead_code)]
struct Person {
    id: u64,
    #[sim(state, input(from = "infected"), output(to = "event"))]
    state: HealthState,
}

#[derive(Fsm, Debug, PartialEq, Clone)]
#[fsm(name = "HealthState")]
#[allow(dead_code)]
enum HealthState {
    #[fsm(trans(
    cond = "Msg::Infected",
    to = "Exposed(uniform(3..-=6))",
    event = "Event::Infected"
    ))]
    Susceptible,
    #[fsm(trans(
    cond = "days_left==1",
    to = "Infectious(uniform(7..-=15))",
    event = "Event::Illness"
    ))]
    #[fsm(rotate(val = "days_left-=1"))]
    Exposed { days_left: usize },
    #[fsm(trans(
    cond = "days_left==1",
    to = "Immune(uniform(30..-=70))",
    event = "Event::Cure"
    ))]
    #[fsm(rotate(
    val = "days_left-=1",
    event = "Event::Contact{count:5, probability:10}"
    ))]
    Infectious { days_left: usize },
    #[fsm(trans(
    cond = "days_left==1",
    to = "Susceptible",
    event = "Event::LostImmunity"
    ))]
    #[fsm(rotate(val = "days_left-=1"))]
    Immune { days_left: usize },
}

#[derive(Simuples)]
#[sim(
name = "BassDiffusion",
method = "composited",
input_name = "BassDiffusionInput",
output_name = "BassDiffusionOutput"
)]
#[allow(dead_code)]
pub struct BassDiffusion {
    #[sim(model(input = "PopulationInput", output = "PopulationOutput"))]
    #[sim(output(from = "total", to = "current_population", ty = "f64"))]
    population: Population,

    #[sim(model(input = "BassInput", output = "BassOutput"))]
    #[sim(output(from = "clients", to = "clients", ty = "f64"))]
    #[sim(mapping(from = "population.total", to = "total_population"))]
    //会自动替换成对应的 Input or Output
    bass: Bass,
}

#[derive(Simuples, Debug, Clone)]
#[allow(dead_code)]
#[sim(
name = "Population",
method = "system_dynamics",
ode_solver = "eula",
input_name = "PopulationInput",
output_name = "PopulationOutput"
)]
pub struct Population {
    #[sim(param(val = "100_000_0000f64"))]
    initial_population: f64,
    #[sim(param(val = "0.002_f64"))]
    move_in_rate: f64,
    #[sim(param(val = "0.001_f64"))]
    move_out_rate: f64,

    #[sim(flow(
    from = "population",
    val = "population * move_in_rate - population * move_out_rate"
    ))]
    population_change: f64,
    #[sim(stock(val = "initial_population"), output(to = "total"))]
    population: f64,
}

#[derive(Simuples, Debug, Clone)]
#[sim(input(name = "Id", ty = "struct"))]
pub struct Id(String);

#[derive(Simuples, Debug, Clone)]
#[allow(dead_code)]
#[sim(
name = "test",
method = "system_dynamics",
ode_solver = "eula",
input_name = "BassInput",
output_name = "BassOutput"
)]
#[sim(input(name = "BassInput", ty = "struct"))]
#[sim(output(name = "BassOutput", ty = "struct"))]
pub struct Bass {
    //参数，val是默认值，引用输入（input）中的字段
    #[sim(param(val = "10_000_f64"), input(from = "total_population"))]
    total_population: f64,
    #[sim(param(val = "0.015_f64"))]
    ad_effectiveness: f64,
    #[sim(param(val = "100_f64"))]
    contact_rate: f64,
    #[sim(param(val = "0.011_f64"))]
    sales_fraction: f64,

    //变量，val是计算表达式
    #[sim(var(val = "potential_clients * ad_effectiveness"))]
    sales_from_ad: f64,
    #[sim(var(
    val = "clients * contact_rate * sales_fraction * potential_clients / total_population"
    ))]
    sales_from_wom: f64,

    //存量，val是初始值，output是映射到output中的字段
    #[sim(stock(val = "total_population"), output(to = "potential_clients"))]
    potential_clients: f64,
    #[sim(stock, output(to = "clients"))]
    clients: f64,

    //流量 from/to 是 stock，
    #[sim(
    flow(
    from = "potential_clients",
    to = "clients",
    val = "sales_from_ad + sales_from_wom"
    ),
    output(to = "sales")
    )]
    sales: f64,
}

/// 仿真推进行为抽象
pub trait Simulate {
    type I;
    //输入类型
    type O;
    //输出类型
    fn push_one_step(&mut self, progress: &Progress, input: &Self::I) -> Self::O;
}

// 仿真推进信息(tick推进方式）
#[derive(Debug, Copy, Clone)]
pub struct Progress {
    pub step: u64,
    pub initial_time: u64,
    pub tick: u32,
}

impl Default for Progress {
    fn default() -> Self {
        Progress {
            step: u64::default(),
            initial_time: u64::default(),
            tick: u32::default(),
        }
    }
}