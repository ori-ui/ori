use std::fmt::Display;

use ori::prelude::*;

#[derive(Clone, Copy, Debug, Default)]
struct Number {
    value: f64,
    position: Option<i8>,
}

impl Number {
    fn new(value: f64) -> Self {
        Self {
            value,
            position: None,
        }
    }

    fn add_digit(&mut self, digit: u8) {
        let Some(position) = self.position else {
            self.position = Some(1);
            self.value = digit as f64;
            return;
        };

        let sign = self.value.signum();

        if position < 0 {
            let pow = 10.0f64.powi(position as i32);
            self.value += digit as f64 * pow * sign;
            self.position = Some(position - 1);
        } else {
            self.value *= 10.0;
            self.value += digit as f64 * sign;
            self.position = Some(position + 1);
        }
    }

    fn remove_digit(&mut self) {
        let Some(position) = self.position else {
            self.position = Some(0);
            self.value = 0.0;
            return;
        };

        if position < -1 {
            self.value *= 10.0f64.powi(-position as i32 - 2);
            self.value = self.value.trunc();
            self.value /= 10.0f64.powi(-position as i32 - 2);

            if position == -2 {
                self.position = Some(0);
            } else {
                self.position = Some(position + 1);
            }
        } else if position >= 0 {
            self.value /= 10.0;
            self.value = self.value.trunc();

            self.position = Some((position - 1).max(0));
        } else {
            self.position = Some(0);
        }

        // ensure that -0.0 is not displayed
        if self.value == -0.0 {
            self.value = 0.0;
        }
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Some(position) = self.position else {
            return write!(f, "{}", self.value);
        };

        if position == -1 {
            write!(f, "{}.", self.value)
        } else if position < 0 {
            write!(f, "{:.1$}", self.value, -position as usize - 1)
        } else {
            write!(f, "{}", self.value)
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Operator {
    None,
    Add,
    Subtract,
    Multiply,
    Divide,
}

fn result_bar(
    cx: Scope,
    operator: Signal<Operator>,
    result: Signal<Number>,
    rhs: Signal<Number>,
) -> impl View {
    let text = cx.memo(move || {
        let result = result.get();
        let operator = operator.get();
        let rhs = rhs.get();

        match operator {
            Operator::None => format!("{}", result),
            Operator::Add => format!("{} + {}", result, rhs),
            Operator::Subtract => format!("{} - {}", result, rhs),
            Operator::Multiply => format!("{} × {}", result, rhs),
            Operator::Divide => format!("{} ÷ {}", result, rhs),
        }
    });

    view! {
        <Div class="result-bar">
            <Text class="result" text=text.get() />
        </Div>
    }
}

fn bar0(
    cx: Scope,
    operator: Signal<Operator>,
    result: Signal<Number>,
    rhs: Signal<Number>,
) -> impl View {
    let clear_all = move |_: &PointerEvent| {
        operator.set(Operator::None);
        result.set(Number::new(0.0));
        rhs.set(Number::new(0.0));
    };

    let clear = move |_: &PointerEvent| {
        if matches!(operator.get(), Operator::None) {
            result.set(Number::new(0.0));
        } else {
            rhs.set(Number::new(0.0));
        }
    };

    let remove_digit = move |_: &PointerEvent| {
        if matches!(operator.get(), Operator::None) {
            //result.modify().remove_digit();
        } else {
            //rhs.modify().remove_digit();
        }
    };

    let divide = move |_: &PointerEvent| {
        operator.set(Operator::Divide);
    };

    view! {
        <Div class="buttons row">
            <Button on:press=clear_all>
                <Text text="CE" />
            </Button>
            <Button on:press=clear>
                <Text text="C" />
            </Button>
            <Button on:press=remove_digit>
                <Text text="\u{e14a}" style:font="icon" />
            </Button>
            <Button on:press=divide>
                <Text text="÷" />
            </Button>
        </Div>
    }
}

fn add_digit(
    operator: Signal<Operator>,
    result: Signal<Number>,
    rhs: Signal<Number>,
    digit: u8,
) -> impl Fn(&PointerEvent) {
    move |_| {
        if matches!(operator.get(), Operator::None) {
            //result.modify().add_digit(digit);
        } else {
            //rhs.modify().add_digit(digit);
        }
    }
}

fn bar1(
    cx: Scope,
    operator: Signal<Operator>,
    result: Signal<Number>,
    rhs: Signal<Number>,
) -> impl View {
    let multiply = move |_: &PointerEvent| {
        operator.set(Operator::Multiply);
    };

    view! {
        <Div class="buttons row">
            <Button class="number" on:press=add_digit(operator, result, rhs, 7)>
                <Text text="7" />
            </Button>
            <Button class="number" on:press=add_digit(operator, result, rhs, 8)>
                <Text text="8" />
            </Button>
            <Button class="number" on:press=add_digit(operator, result, rhs, 9)>
                <Text text="9" />
            </Button>
            <Button on:press=multiply>
                <Text text="×" />
            </Button>
        </Div>
    }
}

fn bar2(
    cx: Scope,
    operator: Signal<Operator>,
    result: Signal<Number>,
    rhs: Signal<Number>,
) -> impl View {
    let subtract = move |_: &PointerEvent| {
        operator.set(Operator::Subtract);
    };

    view! {
        <Div class="buttons row">
            <Button class="number" on:press=add_digit(operator, result, rhs, 4)>
                <Text text="4" />
            </Button>
            <Button class="number" on:press=add_digit(operator, result, rhs, 5)>
                <Text text="5" />
            </Button>
            <Button class="number" on:press=add_digit(operator, result, rhs, 6)>
                <Text text="6" />
            </Button>
            <Button on:press=subtract>
                <Text text="-" />
            </Button>
        </Div>
    }
}

fn bar3(
    cx: Scope,
    operator: Signal<Operator>,
    result: Signal<Number>,
    rhs: Signal<Number>,
) -> impl View {
    let add = move |_: &PointerEvent| {
        operator.set(Operator::Add);
    };

    view! {
        <Div class="buttons row">
            <Button class="number" on:press=add_digit(operator, result, rhs, 1)>
                <Text text="1" />
            </Button>
            <Button class="number" on:press=add_digit(operator, result, rhs, 2)>
                <Text text="2" />
            </Button>
            <Button class="number" on:press=add_digit(operator, result, rhs, 3)>
                <Text text="3" />
            </Button>
            <Button on:press=add>
                <Text text="+" />
            </Button>
        </Div>
    }
}

fn bar4(
    cx: Scope,
    operator: Signal<Operator>,
    result: Signal<Number>,
    rhs: Signal<Number>,
) -> impl View {
    let negate = move |_: &PointerEvent| {
        if result.get().value == 0.0 {
            return;
        }

        if matches!(operator.get(), Operator::None) {
            //result.modify().value *= -1.0;
        } else {
            //rhs.modify().value *= -1.0;
        }
    };

    let add_point = move |_: &PointerEvent| {
        if let Some(position) = result.get().position {
            if position < 0 {
                return;
            }
        }

        if matches!(operator.get(), Operator::None) {
            //result.modify().position = Some(-1);
        } else {
            //rhs.modify().position = Some(-1);
        }
    };

    let equals = |_: &PointerEvent| {
        //let mut result = result.modify();
        //let mut rhs = rhs.modify();
        //let mut operator = operator.modify();
        //match *operator {
        //    Operator::None => {}
        //    Operator::Add => {
        //        *result = Number::new(result.value + rhs.value);
        //    }
        //    Operator::Subtract => {
        //        *result = Number::new(result.value - rhs.value);
        //    }
        //    Operator::Multiply => {
        //        *result = Number::new(result.value * rhs.value);
        //    }
        //    Operator::Divide => {
        //        *result = Number::new(result.value / rhs.value);
        //    }
        //}
        //*operator = Operator::None;
        //*rhs = Number::new(0.0);
    };

    view! {
        <Div class="buttons row">
            <Button on:press=negate>
                <Text text="±" />
            </Button>
            <Button class="number" on:press=add_digit(operator, result, rhs, 0)>
                <Text text="0" />
            </Button>
            <Button on:press=add_point>
                <Text text="." />
            </Button>
            <Button on:press=equals>
                <Text text="=" />
            </Button>
        </Div>
    }
}

fn buttons(
    cx: Scope,
    operator: Signal<Operator>,
    result: Signal<Number>,
    rhs: Signal<Number>,
) -> impl View {
    view! {
        <Div class="buttons column">
            { bar0(cx, operator, result, rhs) }
            { bar1(cx, operator, result, rhs) }
            { bar2(cx, operator, result, rhs) }
            { bar3(cx, operator, result, rhs) }
            { bar4(cx, operator, result, rhs) }
        </Div>
    }
}

fn ui(cx: Scope) -> impl View {
    let operator = cx.signal(Operator::None);
    let result = cx.signal(Number::new(0.0));
    let rhs = cx.signal(Number::new(0.0));

    view! {
        { result_bar(cx, operator, result, rhs) }
        { buttons(cx, operator, result, rhs) }
    }
}

fn main() {
    App::new(ui)
        .title("Calculator (examples/calculator.rs)")
        .style("examples/style/calculator.css")
        .reziseable(false)
        .transparent()
        .size(300.0, 400.0)
        .run();
}
