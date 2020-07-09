package main

import "fmt"

type Combinator interface {
	apply(Combinator) Combinator
	data() interface{}
}

type Substitution struct {
	x *Combinator
	y *Combinator
}

func (s Substitution) data() interface{} { return nil }

func (s Substitution) apply(arg Combinator) Combinator {
	if s.x == nil {
		s.x = &arg
	} else if s.y == nil {
		s.y = &arg
	} else {
		return (*s.x).apply(arg).apply((*s.y).apply(arg))
	}

	return s
}

func (s Substitution) String() string {
	if s.x == nil {
		return "S"
	} else if s.y == nil {
		return fmt.Sprintf("S(%v)", *s.x)
	} else {
		return fmt.Sprintf("S(%v)(%v)", *s.x, *s.y)
	}
}

type Constant struct {
	x *Combinator
}

func (k Constant) data() interface{} { return nil }

func (k Constant) apply(arg Combinator) Combinator {
	if k.x == nil {
		k.x = &arg
	} else {
		return *k.x
	}

	return k
}

func (k Constant) String() string {
	if k.x == nil {
		return "K"
	} else {
		return fmt.Sprintf("K(%v)", *k.x)
	}
}

type Identity struct{}

func (i Identity) data() interface{} { return nil }

func (i Identity) apply(arg Combinator) Combinator {
	return arg
}

func (i Identity) String() string {
	return "I"
}

type Table struct {
	table map[string]Combinator
}

func make_table(table map[string]Combinator) Table {
	return Table{table}
}

func (t Table) data() interface{} { return t.table }

func (t Table) apply(arg Combinator) Combinator {
	return t
}

func (t Table) String() string {
	result := "{ "

	for k, v := range t.table {
		result += fmt.Sprintf("\"%v\":%v ", k, v)
	}

	return result + "}"
}

type List struct {
	list []Combinator
}

func make_list(list []Combinator) List {
	return List{list}
}

func (l List) data() interface{} { return l.list }

func (l List) apply(arg Combinator) Combinator {
	return l
}

func (l List) String() string {
	result := "[ "

	for _, v := range l.list {
		result += fmt.Sprintf("%v ", v)
	}

	return result + "]"
}

type Str struct {
	s string
}

func make_str(s string) Str {
	return Str{s}
}

func (s Str) data() interface{} { return s.s }

func (s Str) apply(arg Combinator) Combinator {
	return s
}

func (s Str) String() string {
	return s.s
}

type I32 struct {
	n int
}

func make_i32(n int) I32 {
	return I32{n}
}

func (i I32) data() interface{} { return i.n }

func (i I32) apply(arg Combinator) Combinator {
	return i
}

func (i I32) String() string {
	return fmt.Sprintf("%v", i.n)
}

type F64 struct {
	n float64
}

func make_f64(n float64) F64 {
	return F64{n}
}

func (f F64) data() interface{} { return f.n }

func (f F64) apply(arg Combinator) Combinator {
	return f
}

func (f F64) String() string {
	return fmt.Sprintf("%v", f.n)
}

type Foreign struct {
	name string
	f    func(Combinator) Combinator
}

func make_foreign(s string, f func(Combinator) Combinator) Foreign {
	return Foreign{s, f}
}

func (f Foreign) data() interface{} { return f.f }

func (f Foreign) apply(arg Combinator) Combinator {
	return f.f(arg)
}

func (f Foreign) String() string {
	return f.name
}

var S = Substitution{nil, nil}
var K = Constant{nil}
var I = Identity{}

func print_f(arg Combinator) Combinator {
	fmt.Print(arg)
	return make_foreign("print", print_f)
}

func recurse_f(x Combinator) Combinator {
	// return make_foreign("recurse", func(x Combinator) Combinator {
	// 	return make_foreign("recurse", func(v Combinator) Combinator {
	// 		return x.apply(x).apply(v)
	// 	})
	// }).apply(make_foreign("recurse", func(x Combinator) Combinator {
	// 		return make_foreign("recurse", func(v Combinator) Combinator {
	// 			return x.apply(x).apply(v)
	// 		})
	// 	}))
	return make_foreign("y.y(x x y)", func(y Combinator) Combinator {
		return y.apply(x.apply(x).apply(y))
	}).apply(make_foreign("recurse", recurse_f))
}

var print = make_foreign("print", print_f)
var recurse = make_foreign("recurse", recurse_f)


type Sub struct {}

func (f Sub) data() interface{} { return nil }

func (f Sub) apply(x Combinator) Combinator {
	return make_foreign("sub", func(y Combinator) Combinator {
		return make_f64(x.data().(float64) - y.data().(float64))
	})
}

type Mul struct {}

func (f Mul) data() interface{} { return nil }

func (f Mul) apply(x Combinator) Combinator {
	return make_foreign("sub", func(y Combinator) Combinator {
		return make_f64(x.data().(float64) * y.data().(float64))
	})
}

type IfThen struct {}

func (f IfThen) data() interface{} { return nil }

func (f IfThen) apply(cond Combinator) Combinator {
	return make_foreign("if", func(x Combinator) Combinator {
		return make_foreign("if", func(y Combinator) Combinator {
			return cond.apply(I).apply(x).apply(y).apply(I)
		})
	})
}

var sub = Sub {}
var mul = Mul {}
var If = IfThen {}
var greater = make_foreign("greater", func(x Combinator) Combinator {
	return make_foreign("greater", func(y Combinator) Combinator {
		if x.data().(float64) > y.data().(float64) {
			return K
		} else {
			return K.apply(I)
		}
	})
})

func main() {
	// fmt.Println(S.apply(K).apply(K).apply(I))
