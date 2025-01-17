use arbitrary::{Arbitrary, Result, Unstructured};
use std::fmt;
use std::iter::once;
use std::ops::ControlFlow;

pub const DATA_TRIG: &str = "
@prefix : <http://example.com/> .

:1 :2 :3 , :4 ;
   :5 true , 1 , 1.0 , 1e0 .

:3 :2 :4 ;
   :5 false , 0 , 0.0 , 0e0 .
";

const NUMBER_OF_NAMED_NODES: u8 = 5;
const NUMBER_OF_VARIABLES: u8 = 4;
const LITERALS: [&str; 11] = [
    "\"foo\"",
    "\"foo\"^^<http://www.w3.org/2001/XMLSchema#string>",
    "\"foo\"@en",
    "true",
    "false",
    "0",
    "0.0",
    "0e0",
    "1",
    "1.0",
    "1e0",
];

pub struct Query {
    inner: QueryContent,
}

#[derive(Arbitrary)]
struct QueryContent {
    // [1]  	QueryUnit	  ::=  	Query
    // [2]  	Query	  ::=  	Prologue ( SelectQuery | ConstructQuery | DescribeQuery | AskQuery ) ValuesClause
    variant: QueryVariant,
    values_clause: ValuesClause,
}

#[derive(Arbitrary)]
enum QueryVariant {
    Select(SelectQuery),
    //TODO: Other variants!
}

impl<'a> Arbitrary<'a> for Query {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        Ok(Self {
            inner: QueryContent::arbitrary(u)?,
        })
    }

    fn arbitrary_take_rest(u: Unstructured<'a>) -> Result<Self> {
        Ok(Self {
            inner: QueryContent::arbitrary_take_rest(u)?,
        })
    }

    fn size_hint(_depth: usize) -> (usize, Option<usize>) {
        (20, None)
    }
}

impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.inner.variant {
            QueryVariant::Select(s) => write!(f, "{s}"),
        }?;
        write!(f, "{}", self.inner.values_clause)
    }
}

impl fmt::Debug for Query {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[derive(Arbitrary)]
struct SelectQuery {
    // [7]  	SelectQuery	  ::=  	SelectClause DatasetClause* WhereClause SolutionModifier
    select_clause: SelectClause,
    where_clause: WhereClause,
    solution_modifier: SolutionModifier,
}

impl fmt::Display for SelectQuery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.select_clause, self.where_clause, self.solution_modifier
        )
    }
}

#[derive(Arbitrary)]
struct SubSelect {
    // [8]  	SubSelect	  ::=  	SelectClause WhereClause SolutionModifier ValuesClause
    select_clause: SelectClause,
    where_clause: WhereClause,
    solution_modifier: SolutionModifier,
    values_clause: ValuesClause,
}

impl fmt::Display for SubSelect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}{}",
            self.select_clause, self.where_clause, self.solution_modifier, self.values_clause
        )
    }
}

#[derive(Arbitrary)]
struct SelectClause {
    // [9]  	SelectClause	  ::=  	'SELECT' ( 'DISTINCT' | 'REDUCED' )? ( ( Var | ( '(' Expression 'AS' Var ')' ) )+ | '*' )
    option: Option<SelectOption>,
    values: SelectValues,
}

#[derive(Arbitrary)]
enum SelectOption {
    Distinct,
    Reduced,
}

#[derive(Arbitrary)]
enum SelectValues {
    Star,
    Projection {
        start: SelectProjection,
        others: Vec<SelectProjection>,
    },
}

#[derive(Arbitrary)]
enum SelectProjection {
    Variable(Var),
    Projection(Expression, Var),
}

impl fmt::Display for SelectClause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SELECT")?;
        if let Some(option) = &self.option {
            match option {
                SelectOption::Distinct => write!(f, " DISTINCT"),
                SelectOption::Reduced => write!(f, " REDUCED"),
            }?;
        }
        match &self.values {
            SelectValues::Star => write!(f, " *"),
            SelectValues::Projection { start, others } => {
                for e in once(start).chain(others) {
                    match e {
                        SelectProjection::Variable(v) => write!(f, " {v}"),
                        SelectProjection::Projection(e, v) => write!(f, " ({e} AS {v})"),
                    }?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Arbitrary)]
struct WhereClause {
    // [17]  	WhereClause	  ::=  	'WHERE'? GroupGraphPattern
    with_where: bool,
    group_graph_pattern: GroupGraphPattern,
}

impl fmt::Display for WhereClause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.with_where {
            write!(f, " WHERE ")?;
        }
        write!(f, "{}", self.group_graph_pattern)
    }
}

#[derive(Arbitrary)]
struct SolutionModifier {
    // [18]  	SolutionModifier	  ::=  	GroupClause? HavingClause? OrderClause? LimitOffsetClauses?
    group: Option<GroupClause>,
    having: Option<HavingClause>,
    #[cfg(feature = "order")]
    order: Option<OrderClause>,
    #[cfg(feature = "limit-offset")]
    limit_offset: Option<LimitOffsetClauses>,
}

impl fmt::Display for SolutionModifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(group) = &self.group {
            write!(f, " {group}")?;
        }
        if let Some(having) = &self.having {
            write!(f, " {having}")?;
        }
        #[cfg(feature = "order")]
        if let Some(order) = &self.order {
            write!(f, " {order}")?;
        }
        #[cfg(feature = "limit-offset")]
        if let Some(limit_offset) = &self.limit_offset {
            write!(f, " {limit_offset}")?;
        }
        Ok(())
    }
}

#[derive(Arbitrary)]
struct GroupClause {
    // [19]  	GroupClause	  ::=  	'GROUP' 'BY' GroupCondition+
    start: GroupCondition,
    others: Vec<GroupCondition>,
}

impl fmt::Display for GroupClause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GROUP BY {}", self.start)?;
        for o in &self.others {
            write!(f, " {o}")?;
        }
        Ok(())
    }
}

#[derive(Arbitrary)]
enum GroupCondition {
    // [20]  	GroupCondition	  ::=  	BuiltInCall | FunctionCall | '(' Expression ( 'AS' Var )? ')' | Var
    BuiltInCall(BuiltInCall),
    // TODO FunctionCall(FunctionCall)
    Projection(Expression, Option<Var>),
    Var(Var),
}

impl fmt::Display for GroupCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BuiltInCall(c) => write!(f, "{c}"),
            //Self::FunctionCall(c) => write!(f, "{}", c),
            Self::Projection(e, v) => {
                if let Some(v) = v {
                    write!(f, "({e} AS {v})")
                } else {
                    write!(f, "({e})")
                }
            }
            Self::Var(v) => write!(f, "{v}"),
        }
    }
}

#[derive(Arbitrary)]
struct HavingClause {
    // [21]  	HavingClause	  ::=  	'HAVING' HavingCondition+
    start: HavingCondition,
    others: Vec<HavingCondition>,
}

impl fmt::Display for HavingClause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HAVING {}", self.start)?;
        for o in &self.others {
            write!(f, " {o}")?;
        }
        Ok(())
    }
}

// [22]  	HavingCondition	  ::=  	Constraint
type HavingCondition = Constraint;

#[cfg(feature = "order")]
#[derive(Arbitrary)]
struct OrderClause {
    // [23]  	OrderClause	  ::=  	'ORDER' 'BY' OrderCondition+
    start: OrderCondition,
    others: Vec<OrderCondition>,
}

#[cfg(feature = "order")]
impl fmt::Display for OrderClause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ORDER BY {}", self.start)?;
        for other in &self.others {
            write!(f, "  {other}")?;
        }
        Ok(())
    }
}

#[cfg(feature = "order")]
#[derive(Arbitrary)]
enum OrderCondition {
    // [24]  	OrderCondition	  ::=  	( ( 'ASC' | 'DESC' ) BrackettedExpression ) | ( Constraint | Var )
    BrackettedExpression {
        is_asc: bool,
        inner: BrackettedExpression,
    },
    Constraint(Constraint),
    Var(Var),
}

#[cfg(feature = "order")]
impl fmt::Display for OrderCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BrackettedExpression { is_asc, inner } => {
                if *is_asc {
                    write!(f, "ASC{inner}")
                } else {
                    write!(f, "DESC{inner}")
                }
            }
            Self::Constraint(c) => write!(f, "{c}"),
            Self::Var(v) => write!(f, "{v}"),
        }
    }
}

#[cfg(feature = "limit-offset")]
#[derive(Arbitrary)]
enum LimitOffsetClauses {
    // [25]  	LimitOffsetClauses	  ::=  	LimitClause OffsetClause? | OffsetClause LimitClause?
    LimitOffset(LimitClause, Option<OffsetClause>),
    OffsetLimit(OffsetClause, Option<LimitClause>),
}

#[cfg(feature = "limit-offset")]
impl fmt::Display for LimitOffsetClauses {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LimitOffset(l, Some(o)) => write!(f, "{l} {o}"),
            Self::LimitOffset(l, None) => write!(f, "{l}"),
            Self::OffsetLimit(o, Some(l)) => write!(f, "{o} {l}"),
            Self::OffsetLimit(o, None) => write!(f, "{o}"),
        }
    }
}

#[cfg(feature = "limit-offset")]
#[derive(Arbitrary)]
struct LimitClause {
    // [26]  	LimitClause	  ::=  	'LIMIT' INTEGER
    value: u8,
}

#[cfg(feature = "limit-offset")]
impl fmt::Display for LimitClause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LIMIT {}", self.value)
    }
}

#[cfg(feature = "limit-offset")]
#[derive(Arbitrary)]
struct OffsetClause {
    // [27]  	OffsetClause	  ::=  	'OFFSET' INTEGER
    value: u8,
}

#[cfg(feature = "limit-offset")]
impl fmt::Display for OffsetClause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OFFSET {}", self.value)
    }
}

#[derive(Arbitrary)]
struct ValuesClause {
    // [28]  	ValuesClause	  ::=  	( 'VALUES' DataBlock )?
    value: Option<DataBlock>,
}

impl fmt::Display for ValuesClause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(value) = &self.value {
            write!(f, " VALUES {value}")
        } else {
            Ok(())
        }
    }
}

#[derive(Arbitrary)]
enum GroupGraphPattern {
    // [53]  	GroupGraphPattern	  ::=  	'{' ( SubSelect | GroupGraphPatternSub ) '}'
    GroupGraphPatternSub(GroupGraphPatternSub),
    SubSelect(Box<SubSelect>),
}

impl fmt::Display for GroupGraphPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " {{ ")?;
        match self {
            Self::GroupGraphPatternSub(p) => write!(f, "{p}"),
            Self::SubSelect(s) => write!(f, "{s}"),
        }?;
        write!(f, " }} ")
    }
}

#[derive(Arbitrary)]
struct GroupGraphPatternSub {
    // [54]  	GroupGraphPatternSub	  ::=  	TriplesBlock? ( GraphPatternNotTriples '.'? TriplesBlock? )*
    start: Option<TriplesBlock>,
    others: Vec<GroupGraphPatternSubOtherBlock>,
}

#[derive(Arbitrary)]
struct GroupGraphPatternSubOtherBlock {
    start: GraphPatternNotTriples,
    with_dot: bool,
    end: Option<TriplesBlock>,
}

impl fmt::Display for GroupGraphPatternSub {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(start) = &self.start {
            write!(f, "{start}")?;
        }
        for other in &self.others {
            write!(f, "{}", other.start)?;
            if other.with_dot {
                write!(f, " . ")?;
            }
            if let Some(end) = &other.end {
                write!(f, "{end}")?;
            }
        }
        Ok(())
    }
}

#[derive(Arbitrary)]
struct TriplesBlock {
    // [55]  	TriplesBlock	  ::=  	TriplesSameSubjectPath ( '.' TriplesBlock? )?
    start: TriplesSameSubjectPath,
    end: Option<Option<Box<TriplesBlock>>>,
}

impl fmt::Display for TriplesBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.start)?;
        if let Some(end) = &self.end {
            write!(f, " . ")?;
            if let Some(end) = end {
                write!(f, "{end}")?;
            }
        }
        Ok(())
    }
}

#[derive(Arbitrary)]
enum GraphPatternNotTriples {
    // [56]  	GraphPatternNotTriples	  ::=  	GroupOrUnionGraphPattern | OptionalGraphPattern | MinusGraphPattern | GraphGraphPattern | ServiceGraphPattern | Filter | Bind | InlineData
    GroupOrUnion(GroupOrUnionGraphPattern),
    Optional(OptionalGraphPattern),
    Minus(MinusGraphPattern),
    Graph(GraphGraphPattern),
    Filter(Filter),
    Bind(Bind),
    InlineData(InlineData), // TODO: ServiceGraphPattern
    #[cfg(feature = "sep-0006")]
    Lateral(LateralGraphPattern),
}

impl fmt::Display for GraphPatternNotTriples {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GroupOrUnion(p) => write!(f, "{p}"),
            Self::Optional(p) => write!(f, "{p}"),
            Self::Minus(p) => write!(f, "{p}"),
            Self::Graph(p) => write!(f, "{p}"),
            Self::Filter(p) => write!(f, "{p}"),
            Self::Bind(p) => write!(f, "{p}"),
            Self::InlineData(p) => write!(f, "{p}"),
            #[cfg(feature = "sep-0006")]
            Self::Lateral(p) => write!(f, "{p}"),
        }
    }
}

#[derive(Arbitrary)]
struct OptionalGraphPattern {
    // [57]  	OptionalGraphPattern	  ::=  	'OPTIONAL' GroupGraphPattern
    inner: GroupGraphPattern,
}

impl fmt::Display for OptionalGraphPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " OPTIONAL {}", self.inner)
    }
}

#[derive(Arbitrary)]
struct LateralGraphPattern {
    // []  	LateralGraphPattern	  ::=  	'LATERAL' GroupGraphPattern
    inner: GroupGraphPattern,
}

impl fmt::Display for LateralGraphPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " LATERAL {}", self.inner)
    }
}

#[derive(Arbitrary)]
struct GraphGraphPattern {
    // [58]  	GraphGraphPattern	  ::=  	'GRAPH' VarOrIri GroupGraphPattern
    graph: VarOrIri,
    inner: GroupGraphPattern,
}

impl fmt::Display for GraphGraphPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " GRAPH {} {}", self.graph, self.inner)
    }
}

#[derive(Arbitrary)]
struct Bind {
    // [60]  	Bind	  ::=  	'BIND' '(' Expression 'AS' Var ')'
    expression: Expression,
    var: Var,
}

impl fmt::Display for Bind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " BIND({} AS {})", self.expression, self.var)
    }
}

#[derive(Arbitrary)]
struct InlineData {
    // [61]  	InlineData	  ::=  	'VALUES' DataBlock
    inner: DataBlock,
}

impl fmt::Display for InlineData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VALUES {}", &self.inner)
    }
}

#[derive(Arbitrary)]
enum DataBlock {
    // [62]  	DataBlock	  ::=  	InlineDataOneVar | InlineDataFull
    OneVar(InlineDataOneVar),
    Full(InlineDataFull),
}

impl fmt::Display for DataBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OneVar(e) => write!(f, "{e}"),
            Self::Full(c) => write!(f, "{c}"),
        }
    }
}

#[derive(Arbitrary)]
struct InlineDataOneVar {
    // [63]  	InlineDataOneVar	  ::=  	Var '{' DataBlockValue* '}'
    var: Var,
    values: Vec<DataBlockValue>,
}

impl fmt::Display for InlineDataOneVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {{", self.var)?;
        for v in &self.values {
            write!(f, " {v}")?;
        }
        write!(f, " }}")
    }
}

struct InlineDataFull {
    // [64]  	InlineDataFull	  ::=  	( NIL | '(' Var* ')' ) '{' ( '(' DataBlockValue* ')' | NIL )* '}'
    vars: Vec<Var>,
    values: Vec<Vec<DataBlockValue>>,
}

impl<'a> Arbitrary<'a> for InlineDataFull {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let vars = u.arbitrary_iter()?.collect::<Result<Vec<_>>>()?;

        let mut values = Vec::new();
        u.arbitrary_loop(Some(0), Some(3), |u| {
            let mut row = Vec::with_capacity(vars.len());
            u.arbitrary_loop(
                Some(vars.len().try_into().unwrap()),
                Some(vars.len().try_into().unwrap()),
                |u| {
                    row.push(u.arbitrary()?);
                    Ok(ControlFlow::Continue(()))
                },
            )?;
            values.push(row);
            Ok(ControlFlow::Continue(()))
        })?;

        Ok(Self { vars, values })
    }
}

impl fmt::Display for InlineDataFull {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "( ")?;
        for v in &self.vars {
            write!(f, " {v}")?;
        }
        write!(f, " ) {{")?;
        for vs in &self.values {
            write!(f, " (")?;
            for v in vs {
                write!(f, " {v}")?;
            }
            write!(f, " )")?;
        }
        write!(f, " }}")
    }
}

#[derive(Arbitrary)]
enum DataBlockValue {
    // [65]  	DataBlockValue	  ::=  	iri | RDFLiteral | NumericLiteral | BooleanLiteral | 'UNDEF'
    Iri(Iri),
    Literal(Literal),
    Undef,
}

impl fmt::Display for DataBlockValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Iri(i) => write!(f, "{i}"),
            Self::Literal(l) => write!(f, "{l}"),
            Self::Undef => write!(f, "UNDEF"),
        }
    }
}

#[derive(Arbitrary)]
struct MinusGraphPattern {
    // [66]  	MinusGraphPattern	  ::=  	'MINUS' GroupGraphPattern
    inner: GroupGraphPattern,
}

impl fmt::Display for MinusGraphPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " MINUS {}", self.inner)
    }
}

#[derive(Arbitrary)]
struct GroupOrUnionGraphPattern {
    // [67]  	GroupOrUnionGraphPattern	  ::=  	GroupGraphPattern ( 'UNION' GroupGraphPattern )*
    start: GroupGraphPattern,
    others: Vec<GroupGraphPattern>,
}

impl fmt::Display for GroupOrUnionGraphPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.start)?;
        for other in &self.others {
            write!(f, " UNION {other}")?;
        }
        Ok(())
    }
}

#[derive(Arbitrary)]
struct Filter {
    // [68]  	Filter	  ::=  	'FILTER' Constraint
    constraint: Constraint,
}

impl fmt::Display for Filter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FILTER {}", self.constraint)
    }
}

#[derive(Arbitrary)]
enum Constraint {
    // [69]  	Constraint	  ::=  	BrackettedExpression | BuiltInCall | FunctionCall
    BrackettedExpression(BrackettedExpression),
    BuiltInCall(BuiltInCall),
    // TODO FunctionCall(FunctionCall),
}

impl fmt::Display for Constraint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BrackettedExpression(e) => write!(f, "{e}"),
            Self::BuiltInCall(c) => write!(f, "{c}"),
            //Self::FunctionCall(c) => write!(f, "{}", c),
        }
    }
}

#[derive(Arbitrary)]
struct FunctionCall {
    // [70]  	FunctionCall	  ::=  	iri ArgList
    iri: Iri,
    args: ArgList,
}

impl fmt::Display for FunctionCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.iri, self.args)
    }
}

#[derive(Arbitrary)]
enum ArgList {
    // [71]  	ArgList	  ::=  	NIL | '(' 'DISTINCT'? Expression ( ',' Expression )* ')'
    Nil,
    NotNil {
        // TODO: DISTINCT
        start: Box<Expression>,
        others: Vec<Expression>,
    },
}

impl fmt::Display for ArgList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        if let Self::NotNil { start, others } = self {
            write!(f, "{start}")?;
            for e in others {
                write!(f, ", {e}")?;
            }
        }
        write!(f, ")")
    }
}

#[derive(Arbitrary)]
struct ExpressionList {
    // [72]  	ExpressionList	  ::=  	NIL | '(' Expression ( ',' Expression )* ')'
    inner: Vec<Expression>,
}

impl fmt::Display for ExpressionList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        for (i, e) in self.inner.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{e}")?;
        }
        write!(f, ")")
    }
}

#[derive(Arbitrary)]
struct PropertyListNotEmpty {
    // [77]  	PropertyListNotEmpty	  ::=  	Verb ObjectList ( ';' ( Verb ObjectList )? )*
    start_predicate: Verb,
    start_object: Box<ObjectList>,
    others: Vec<Option<PropertyListElement>>,
}

#[derive(Arbitrary)]
struct PropertyListElement {
    predicate: Verb,
    object: ObjectList,
}

impl fmt::Display for PropertyListNotEmpty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.start_predicate, self.start_object)?;
        for other in &self.others {
            write!(f, " ; ")?;
            if let Some(e) = other {
                write!(f, "{} {}", e.predicate, e.object)?;
            }
        }
        Ok(())
    }
}

#[derive(Arbitrary)]
enum Verb {
    // [78]  	Verb	  ::=  	VarOrIri | 'a'
    VarOrIri(VarOrIri),
    A,
}

impl fmt::Display for Verb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VarOrIri(iri) => write!(f, "{iri}"),
            Self::A => write!(f, " a "),
        }
    }
}

#[derive(Arbitrary)]
struct ObjectList {
    // [79]  	ObjectList	  ::=  	Object ( ',' Object )*
    start: Object,
    others: Vec<Object>,
}

impl fmt::Display for ObjectList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.start)?;
        for other in &self.others {
            write!(f, " , ")?;
            write!(f, "{other}")?;
        }
        Ok(())
    }
}

// [80]  	Object	  ::=  	GraphNode
type Object = GraphNode;

#[derive(Arbitrary)]
enum TriplesSameSubjectPath {
    // [81]  	TriplesSameSubjectPath	  ::=  	VarOrTerm PropertyListPathNotEmpty | TriplesNodePath PropertyListPath
    Atomic {
        subject: VarOrTerm,
        predicate_object: PropertyListPathNotEmpty,
    },
    Other {
        subject: TriplesNodePath,
        predicate_object: PropertyListPath,
    },
}

impl fmt::Display for TriplesSameSubjectPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Atomic {
                subject,
                predicate_object,
            } => {
                write!(f, "{subject}{predicate_object}")
            }
            Self::Other {
                subject,
                predicate_object,
            } => {
                write!(f, "{subject} {predicate_object}")
            }
        }
    }
}

#[derive(Arbitrary)]
struct PropertyListPath {
    // [82]  	PropertyListPath	  ::=  	PropertyListPathNotEmpty?
    inner: Option<PropertyListPathNotEmpty>,
}

impl fmt::Display for PropertyListPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(p) = &self.inner {
            write!(f, "{p}")
        } else {
            Ok(())
        }
    }
}

#[derive(Arbitrary)]
struct PropertyListPathNotEmpty {
    // [83]  	PropertyListPathNotEmpty	  ::=  	( VerbPath | VerbSimple ) ObjectListPath ( ';' ( ( VerbPath | VerbSimple ) ObjectListPath )? )*
    start_predicate: PropertyListPathNotEmptyVerb,
    start_object: Box<ObjectListPath>,
    others: Vec<Option<PropertyListPathElement>>,
}

#[derive(Arbitrary)]
enum PropertyListPathNotEmptyVerb {
    VerbPath(VerbPath),
    VerbSimple(VerbSimple),
}

#[derive(Arbitrary)]
struct PropertyListPathElement {
    predicate: PropertyListPathNotEmptyVerb,
    object: ObjectListPath,
}

impl fmt::Display for PropertyListPathNotEmpty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.start_predicate {
            PropertyListPathNotEmptyVerb::VerbPath(p) => write!(f, "{p}"),
            PropertyListPathNotEmptyVerb::VerbSimple(s) => write!(f, "{s}"),
        }?;
        write!(f, "{}", self.start_object)?;
        for other in &self.others {
            write!(f, " ; ")?;
            if let Some(e) = other {
                match &e.predicate {
                    PropertyListPathNotEmptyVerb::VerbPath(p) => write!(f, "{p}"),
                    PropertyListPathNotEmptyVerb::VerbSimple(s) => write!(f, "{s}"),
                }?;
                write!(f, "{}", e.object)?;
            }
        }
        Ok(())
    }
}

// [84]  	VerbPath	  ::=  	Path
type VerbPath = Path;

// [85]  	VerbSimple	  ::=  	Var
type VerbSimple = Var;

#[derive(Arbitrary)]
struct ObjectListPath {
    // [86]  	ObjectListPath	  ::=  	ObjectPath ( ',' ObjectPath )*
    start: ObjectPath,
    others: Vec<ObjectPath>,
}

impl fmt::Display for ObjectListPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.start)?;
        for other in &self.others {
            write!(f, " , {other}")?;
        }
        Ok(())
    }
}

// [87]  	ObjectPath	  ::=  	GraphNodePath
type ObjectPath = GraphNodePath;

// [88]  	Path	  ::=  	PathAlternative
type Path = PathAlternative;

#[derive(Arbitrary)]
struct PathAlternative {
    // [89]  	PathAlternative	  ::=  	PathSequence ( '|' PathSequence )*
    start: PathSequence,
    others: Vec<PathSequence>,
}

impl fmt::Display for PathAlternative {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.start)?;
        for other in &self.others {
            write!(f, " | {other}")?;
        }
        Ok(())
    }
}

#[derive(Arbitrary)]
struct PathSequence {
    // [90]  	PathSequence	  ::=  	PathEltOrInverse ( '/' PathEltOrInverse )*
    start: PathEltOrInverse,
    others: Vec<PathEltOrInverse>,
}

impl fmt::Display for PathSequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.start)?;
        for other in &self.others {
            write!(f, " / {other}")?;
        }
        Ok(())
    }
}

#[derive(Arbitrary)]
struct PathElt {
    // [91]  	PathElt	  ::=  	PathPrimary PathMod?
    path: PathPrimary,
    mode: Option<PathMod>,
}

impl fmt::Display for PathElt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path)?;
        if let Some(mode) = &self.mode {
            write!(f, "{mode}")?;
        }
        Ok(())
    }
}

#[derive(Arbitrary)]
enum PathEltOrInverse {
    // [92]  	PathEltOrInverse	  ::=  	PathElt | '^' PathElt
    PathElt(PathElt),
    Inverse(PathElt),
}

impl fmt::Display for PathEltOrInverse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PathElt(e) => write!(f, "{e}"),
            Self::Inverse(e) => write!(f, " ^{e}"),
        }
    }
}

#[derive(Arbitrary)]
enum PathMod {
    // [93]  	PathMod	  ::=  	'?' | '*' | '+'
    ZeroOrOne,
    ZeroOrMore,
    OneOrMore,
}

impl fmt::Display for PathMod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroOrOne => write!(f, " ? "),
            Self::ZeroOrMore => write!(f, " * "),
            Self::OneOrMore => write!(f, " + "),
        }
    }
}

#[derive(Arbitrary)]
enum PathPrimary {
    // [94]  	PathPrimary	  ::=  	iri | 'a' | '!' PathNegatedPropertySet | '(' Path ')'
    Iri(Iri),
    A,
    Negated(PathNegatedPropertySet),
    Child(Box<Path>),
}

impl fmt::Display for PathPrimary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Iri(iri) => write!(f, "{iri}"),
            Self::A => write!(f, " a "),
            Self::Negated(n) => write!(f, "!{n}"),
            Self::Child(c) => write!(f, "({c})"),
        }
    }
}

#[derive(Arbitrary)]
enum PathNegatedPropertySet {
    // [95]  	PathNegatedPropertySet	  ::=  	PathOneInPropertySet | '(' ( PathOneInPropertySet ( '|' PathOneInPropertySet )* )? ')'
    Single(PathOneInPropertySet),
    Multiple {
        start: PathOneInPropertySet,
        others: Vec<PathOneInPropertySet>,
    },
}

impl fmt::Display for PathNegatedPropertySet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Single(p) => write!(f, "{p}"),
            Self::Multiple { start, others } => {
                write!(f, " ( {start}")?;
                for other in others {
                    write!(f, " | {other}")?;
                }
                write!(f, " ) ")
            }
        }
    }
}

#[derive(Arbitrary)]
enum PathOneInPropertySet {
    // [96]  	PathOneInPropertySet	  ::=  	iri | 'a' | '^' ( iri | 'a' )
    Iri(Iri),
    A,
    NegatedIri(Iri),
    NegatedA,
}

impl fmt::Display for PathOneInPropertySet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Iri(iri) => write!(f, "{iri}"),
            Self::A => write!(f, " a "),
            Self::NegatedIri(iri) => write!(f, "^{iri}"),
            Self::NegatedA => write!(f, " ^a "),
        }
    }
}

#[derive(Arbitrary)]
enum TriplesNode {
    // [98]  	TriplesNode	  ::=  	Collection | BlankNodePropertyList
    Collection(Collection),
    BlankNodePropertyList(BlankNodePropertyList),
}

impl fmt::Display for TriplesNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Collection(p) => write!(f, "{p}"),
            Self::BlankNodePropertyList(p) => write!(f, "{p}"),
        }
    }
}

#[derive(Arbitrary)]
struct BlankNodePropertyList {
    // [99]  	BlankNodePropertyList	  ::=  	'[' PropertyListNotEmpty ']'
    inner: PropertyListNotEmpty,
}

impl fmt::Display for BlankNodePropertyList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[ {} ]", self.inner)
    }
}

#[derive(Arbitrary)]
enum TriplesNodePath {
    // [100]  	TriplesNodePath	  ::=  	CollectionPath | BlankNodePropertyListPath
    CollectionPath(CollectionPath),
    BlankNodePropertyListPath(BlankNodePropertyListPath),
}

impl fmt::Display for TriplesNodePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CollectionPath(p) => write!(f, "{p}"),
            Self::BlankNodePropertyListPath(p) => write!(f, "{p}"),
        }
    }
}

#[derive(Arbitrary)]
struct BlankNodePropertyListPath {
    // [101]  	BlankNodePropertyListPath	  ::=  	'[' PropertyListPathNotEmpty ']'
    inner: PropertyListPathNotEmpty,
}

impl fmt::Display for BlankNodePropertyListPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[ {} ]", self.inner)
    }
}

#[derive(Arbitrary)]
struct Collection {
    // [102]  	Collection	  ::=  	'(' GraphNode+ ')'
    start: Box<GraphNode>,
    others: Vec<GraphNode>,
}

impl fmt::Display for Collection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "( {}", self.start)?;
        for e in &self.others {
            write!(f, " {e}")?;
        }
        write!(f, " )")
    }
}

#[derive(Arbitrary)]
struct CollectionPath {
    // [103]  	CollectionPath	  ::=  	'(' GraphNodePath+ ')'
    start: Box<GraphNodePath>,
    others: Vec<GraphNodePath>,
}

impl fmt::Display for CollectionPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "( {}", self.start)?;
        for e in &self.others {
            write!(f, " {e}")?;
        }
        write!(f, " )")
    }
}

#[derive(Arbitrary)]
enum GraphNode {
    // [104]  	GraphNode	  ::=  	VarOrTerm | TriplesNode
    VarOrTerm(VarOrTerm),
    TriplesNode(TriplesNode),
}

impl fmt::Display for GraphNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VarOrTerm(t) => write!(f, "{t}"),
            Self::TriplesNode(t) => write!(f, "{t}"),
        }
    }
}

#[derive(Arbitrary)]
enum GraphNodePath {
    // [105]  	GraphNodePath	  ::=  	VarOrTerm | TriplesNodePath
    VarOrTerm(VarOrTerm),
    TriplesNodePath(TriplesNodePath),
}

impl fmt::Display for GraphNodePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VarOrTerm(t) => write!(f, "{t}"),
            Self::TriplesNodePath(p) => write!(f, "{p}"),
        }
    }
}

#[derive(Arbitrary)]
enum VarOrTerm {
    // [106]  	VarOrTerm	  ::=  	Var | GraphTerm
    Var(Var),
    GraphTerm(GraphTerm),
}

impl fmt::Display for VarOrTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Var(v) => write!(f, "{v}"),
            Self::GraphTerm(t) => write!(f, "{t}"),
        }
    }
}

#[derive(Arbitrary)]
enum VarOrIri {
    // [107]  	VarOrIri	  ::=  	Var | iri
    Var(Var),
    Iri(Iri),
}

impl fmt::Display for VarOrIri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Var(v) => write!(f, "{v}"),
            Self::Iri(t) => write!(f, "{t}"),
        }
    }
}

struct Var {
    // [108]  	Var	  ::=  	VAR1 | VAR2
    value: u8,
}

impl Arbitrary<'_> for Var {
    fn arbitrary(u: &mut Unstructured<'_>) -> Result<Self> {
        Ok(Self {
            value: u.int_in_range(1..=NUMBER_OF_VARIABLES)?,
        })
    }

    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        <u8 as Arbitrary>::size_hint(depth)
    }
}

impl fmt::Display for Var {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " ?{} ", self.value)
    }
}

#[derive(Arbitrary)]
enum GraphTerm {
    // [109]  	GraphTerm	  ::=  	iri | RDFLiteral | NumericLiteral | BooleanLiteral | BlankNode | NIL
    Iri(Iri),
    Literal(Literal),
    Nil,
    // TODO: BlankNode
}

impl fmt::Display for GraphTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Iri(iri) => write!(f, "{iri}"),
            Self::Literal(l) => write!(f, "{l}"),
            Self::Nil => write!(f, " () "),
        }
    }
}

// [110]  	Expression	  ::=  	ConditionalOrExpression
type Expression = ConditionalOrExpression;

#[derive(Arbitrary)]
struct ConditionalOrExpression {
    // [111]  	ConditionalOrExpression	  ::=  	ConditionalAndExpression ( '||' ConditionalAndExpression )*
    start: ConditionalAndExpression,
    others: Vec<ConditionalAndExpression>,
}

impl fmt::Display for ConditionalOrExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.start)?;
        for e in &self.others {
            write!(f, " || {e}")?;
        }
        Ok(())
    }
}

#[derive(Arbitrary)]
struct ConditionalAndExpression {
    // [112]  	ConditionalAndExpression	  ::=  	ValueLogical ( '&&' ValueLogical )*
    start: ValueLogical,
    others: Vec<ValueLogical>,
}

impl fmt::Display for ConditionalAndExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.start)?;
        for e in &self.others {
            write!(f, " && {e}")?;
        }
        Ok(())
    }
}

// [113]  	ValueLogical	  ::=  	RelationalExpression
type ValueLogical = RelationalExpression;

#[derive(Arbitrary)]
enum RelationalExpression {
    // [114]  	RelationalExpression	  ::=  	NumericExpression ( '=' NumericExpression | '!=' NumericExpression | '<' NumericExpression | '>' NumericExpression | '<=' NumericExpression | '>=' NumericExpression | 'IN' ExpressionList | 'NOT' 'IN' ExpressionList )?
    Base(NumericExpression),
    Equal(NumericExpression, NumericExpression),
    NotEqual(NumericExpression, NumericExpression),
    Less(NumericExpression, NumericExpression),
    LessOrEqual(NumericExpression, NumericExpression),
    Greater(NumericExpression, NumericExpression),
    GreaterOrEqual(NumericExpression, NumericExpression),
    In(NumericExpression, ExpressionList),
    NotIn(NumericExpression, ExpressionList),
}

impl fmt::Display for RelationalExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Base(e) => write!(f, "{e}"),
            Self::Equal(a, b) => write!(f, "{a} = {b}"),
            Self::NotEqual(a, b) => write!(f, "{a} != {b}"),
            Self::Less(a, b) => write!(f, "{a} < {b}"),
            Self::LessOrEqual(a, b) => write!(f, "{a} <= {b}"),
            Self::Greater(a, b) => write!(f, "{a} > {b}"),
            Self::GreaterOrEqual(a, b) => write!(f, "{a} >= {b}"),
            Self::In(a, b) => write!(f, "{a} IN {b}"),
            Self::NotIn(a, b) => write!(f, "{a} NOT IN {b}"),
        }
    }
}

// [115]  	NumericExpression	  ::=  	AdditiveExpression
type NumericExpression = AdditiveExpression;

#[derive(Arbitrary)]
enum AdditiveExpression {
    // [116]  	AdditiveExpression	  ::=  	MultiplicativeExpression ( '+' MultiplicativeExpression | '-' MultiplicativeExpression | ( NumericLiteralPositive | NumericLiteralNegative ) ( ( '*' UnaryExpression ) | ( '/' UnaryExpression ) )* )*
    Base(MultiplicativeExpression),
    Plus(MultiplicativeExpression, MultiplicativeExpression),
    Minus(MultiplicativeExpression, MultiplicativeExpression), // TODO: Prefix + and -
}

impl fmt::Display for AdditiveExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Base(e) => write!(f, "{e}"),
            Self::Plus(a, b) => write!(f, "{a} + {b}"),
            Self::Minus(a, b) => write!(f, "{a} - {b}"),
        }
    }
}

#[derive(Arbitrary)]
enum MultiplicativeExpression {
    // [117]  	MultiplicativeExpression	  ::=  	UnaryExpression ( '*' UnaryExpression | '/' UnaryExpression )*
    Base(UnaryExpression),
    Mul(UnaryExpression, UnaryExpression),
    Div(UnaryExpression, UnaryExpression),
}

impl fmt::Display for MultiplicativeExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Base(e) => write!(f, "{e}"),
            Self::Mul(a, b) => write!(f, "{a} * {b}"),
            Self::Div(a, b) => write!(f, "{a} / {b}"),
        }
    }
}

#[derive(Arbitrary)]
enum UnaryExpression {
    // [118]  	UnaryExpression	  ::=  	  '!' PrimaryExpression | '+' PrimaryExpression | '-' PrimaryExpression | PrimaryExpression
    Not(PrimaryExpression),
    Plus(PrimaryExpression),
    Minus(PrimaryExpression),
    Base(PrimaryExpression),
}

impl fmt::Display for UnaryExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Not(e) => write!(f, "!{e}"),
            Self::Plus(e) => write!(f, "+{e}"),
            Self::Minus(e) => write!(f, "-{e}"),
            Self::Base(e) => write!(f, "{e}"),
        }
    }
}

#[derive(Arbitrary)]
enum PrimaryExpression {
    // [119]  	PrimaryExpression	  ::=  	BrackettedExpression | BuiltInCall | iriOrFunction | RDFLiteral | NumericLiteral | BooleanLiteral | Var
    Bracketted(BrackettedExpression),
    BuiltInCall(BuiltInCall),
    IriOrFunction(IriOrFunction),
    Literal(Literal),
    Var(Var),
}

impl fmt::Display for PrimaryExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bracketted(e) => write!(f, "{e}"),
            Self::BuiltInCall(e) => write!(f, "{e}"),
            Self::IriOrFunction(e) => write!(f, "{e}"),
            Self::Literal(e) => write!(f, "{e}"),
            Self::Var(e) => write!(f, "{e}"),
        }
    }
}

#[derive(Arbitrary)]
struct BrackettedExpression {
    // [120]  	BrackettedExpression	  ::=  	'(' Expression ')'
    inner: Box<Expression>,
}

impl fmt::Display for BrackettedExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({})", self.inner)
    }
}

#[derive(Arbitrary)]
enum BuiltInCall {
    // [121]  	BuiltInCall	  ::=  	  Aggregate
    // | 'STR' '(' Expression ')'
    // | 'LANG' '(' Expression ')'
    // | 'LANGMATCHES' '(' Expression ',' Expression ')'
    // | 'DATATYPE' '(' Expression ')'
    // | 'BOUND' '(' Var ')'
    // | 'IRI' '(' Expression ')'
    // | 'URI' '(' Expression ')'
    // | 'BNODE' ( '(' Expression ')' | NIL )
    // | 'RAND' NIL
    // | 'ABS' '(' Expression ')'
    // | 'CEIL' '(' Expression ')'
    // | 'FLOOR' '(' Expression ')'
    // | 'ROUND' '(' Expression ')'
    // | 'CONCAT' ExpressionList
    // | SubstringExpression
    // | 'STRLEN' '(' Expression ')'
    // | StrReplaceExpression
    // | 'UCASE' '(' Expression ')'
    // | 'LCASE' '(' Expression ')'
    // | 'ENCODE_FOR_URI' '(' Expression ')'
    // | 'CONTAINS' '(' Expression ',' Expression ')'
    // | 'STRSTARTS' '(' Expression ',' Expression ')'
    // | 'STRENDS' '(' Expression ',' Expression ')'
    // | 'STRBEFORE' '(' Expression ',' Expression ')'
    // | 'STRAFTER' '(' Expression ',' Expression ')'
    // | 'YEAR' '(' Expression ')'
    // | 'MONTH' '(' Expression ')'
    // | 'DAY' '(' Expression ')'
    // | 'HOURS' '(' Expression ')'
    // | 'MINUTES' '(' Expression ')'
    // | 'SECONDS' '(' Expression ')'
    // | 'TIMEZONE' '(' Expression ')'
    // | 'TZ' '(' Expression ')'
    // | 'NOW' NIL
    // | 'UUID' NIL
    // | 'STRUUID' NIL
    // | 'MD5' '(' Expression ')'
    // | 'SHA1' '(' Expression ')'
    // | 'SHA256' '(' Expression ')'
    // | 'SHA384' '(' Expression ')'
    // | 'SHA512' '(' Expression ')'
    // | 'COALESCE' ExpressionList
    // | 'IF' '(' Expression ',' Expression ',' Expression ')'
    // | 'STRLANG' '(' Expression ',' Expression ')'
    // | 'STRDT' '(' Expression ',' Expression ')'
    // | 'sameTerm' '(' Expression ',' Expression ')'
    // | 'isIRI' '(' Expression ')'
    // | 'isURI' '(' Expression ')'
    // | 'isBLANK' '(' Expression ')'
    // | 'isLITERAL' '(' Expression ')'
    // | 'isNUMERIC' '(' Expression ')'
    // | RegexExpression
    // | ExistsFunc
    // | NotExistsFunc
    Str(Box<Expression>),
    Lang(Box<Expression>),
    Datatype(Box<Expression>),
    Bound(Var),
    Iri(Box<Expression>),
    Bnode(Box<Expression>),
    Coalesce(ExpressionList),
    If(Box<Expression>, Box<Expression>, Box<Expression>),
    StrLang(Box<Expression>, Box<Expression>),
    StrDt(Box<Expression>, Box<Expression>),
    SameTerm(Box<Expression>, Box<Expression>),
    IsIri(Box<Expression>),
    IsBlank(Box<Expression>),
    IsLiteral(Box<Expression>),
    IsNumeric(Box<Expression>),
    Exists(ExistsFunc),
    NotExists(NotExistsFunc), //TODO: Other functions
}

impl fmt::Display for BuiltInCall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Str(v) => write!(f, "STR({v})"),
            Self::Lang(v) => write!(f, "LANG({v})"),
            Self::Datatype(v) => write!(f, "DATATYPE({v})"),
            Self::Bound(v) => write!(f, "BOUND({v})"),
            Self::Iri(v) => write!(f, "IRI({v})"),
            Self::Bnode(v) => write!(f, "BNODE({v})"),
            Self::Coalesce(vs) => write!(f, "COALESCE{vs}"),
            Self::If(a, b, c) => write!(f, "IF({a}, {b}, {c})"),
            Self::StrLang(a, b) => write!(f, "STRLANG({a}, {b})"),
            Self::StrDt(a, b) => write!(f, "STRDT({a}, {b})"),
            Self::SameTerm(a, b) => write!(f, "sameTerm({a}, {b})"),
            Self::IsIri(e) => write!(f, "isIRI({e})"),
            Self::IsBlank(e) => write!(f, "isBlank({e})"),
            Self::IsLiteral(e) => write!(f, "isLiteral({e})"),
            Self::IsNumeric(e) => write!(f, "isNumeric({e})"),
            Self::Exists(e) => write!(f, "{e}"),
            Self::NotExists(e) => write!(f, "{e}"),
        }
    }
}

#[derive(Arbitrary)]
struct ExistsFunc {
    // [125]  	ExistsFunc	  ::=  	'EXISTS' GroupGraphPattern
    pattern: GroupGraphPattern,
}

impl fmt::Display for ExistsFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EXISTS {}", self.pattern)
    }
}

#[derive(Arbitrary)]
struct NotExistsFunc {
    // [126]  	NotExistsFunc	  ::=  	'NOT' 'EXISTS' GroupGraphPattern
    pattern: GroupGraphPattern,
}

impl fmt::Display for NotExistsFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NOT EXISTS {}", self.pattern)
    }
}

#[derive(Arbitrary)]
struct IriOrFunction {
    // [128]  	iriOrFunction	  ::=  	iri ArgList?
    iri: Iri,
    //TODO args: Option<ArgList>,
}

impl fmt::Display for IriOrFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.iri)?;
        /*if let Some(args) = &self.args {
            write!(f, "{}", args)?;
        }*/
        Ok(())
    }
}

struct Literal {
    // [129]  	RDFLiteral	  ::=  	String ( LANGTAG | ( '^^' iri ) )?
    // [130]  	NumericLiteral	  ::=  	NumericLiteralUnsigned | NumericLiteralPositive | NumericLiteralNegative
    // [131]  	NumericLiteralUnsigned	  ::=  	INTEGER | DECIMAL | DOUBLE
    // [132]  	NumericLiteralPositive	  ::=  	INTEGER_POSITIVE | DECIMAL_POSITIVE | DOUBLE_POSITIVE
    // [133]  	NumericLiteralNegative	  ::=  	INTEGER_NEGATIVE | DECIMAL_NEGATIVE | DOUBLE_NEGATIVE
    // [134]  	BooleanLiteral	  ::=  	'true' | 'false'
    value: &'static str,
}

impl Arbitrary<'_> for Literal {
    fn arbitrary(u: &mut Unstructured<'_>) -> Result<Self> {
        Ok(Self {
            value: u.choose(LITERALS.as_slice())?,
        })
    }

    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        <u8 as Arbitrary>::size_hint(depth)
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

struct Iri {
    // [136]  	iri	  ::=  	IRIREF | PrefixedName
    value: u8,
}

impl Arbitrary<'_> for Iri {
    fn arbitrary(u: &mut Unstructured<'_>) -> Result<Self> {
        Ok(Self {
            value: u.int_in_range(1..=NUMBER_OF_NAMED_NODES)?,
        })
    }

    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        <u8 as Arbitrary>::size_hint(depth)
    }
}

impl fmt::Display for Iri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " <http://example.org/{}> ", self.value)
    }
}
