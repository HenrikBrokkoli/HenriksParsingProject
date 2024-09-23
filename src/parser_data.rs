use std::collections::HashMap;
use std::fmt;
use std::process::id;
use std::rc::Rc;
use std::str::Chars;
use peekables::{ParseProcess, PeekableWrapper};
use vms::{Instruction, VM};


///Contains the ParseRules, the vector of elements and a Hashmap of ElementData
pub struct ParserData<T> where T: VM {
    pub parse_rules: ParseRules<T>,
    pub element_types: Vec<ElementType>,
    pub element_verbose_map:HashMap<ElementVerbose,ElementIndex>,
    pub element_data: Vec<ElementData>,
}

impl<T> ParserData<T> where T: VM {
    pub fn new() -> ParserData<T> {
        ParserData {
            parse_rules: ParseRules { rules: Default::default(), ignore: None },
            element_types: vec![],
            element_verbose_map: Default::default(),
            element_data: vec![],
        }
    }
    
    pub fn get_element_data(&self, ix: ElementIndex)-> Option<&ElementData>{
        self.element_data.get(ix)
    }
    pub fn get_element(&self, ix: ElementIndex)-> Option<Element>{
        match self.element_types.get(ix){
            None => {None}
            Some(et) => {Some(Element::new(ix,*et))}
        }
    }
    
    pub fn get_element_verbose(&self,ix: ElementIndex)->Option<ElementVerbose>{ 
        let data=self.element_data.get(ix);
        let et=self.element_types.get(ix);
        Some(ElementVerbose::new(data?.name.clone(),*et?))
    }

    pub fn get_elements_verbose(&self)->Vec<ElementVerbose>{
        let mut elements_verbose = vec![];
        for (ix, element_data) in self.element_data.iter().enumerate() {
            elements_verbose.push(ElementVerbose::new(element_data.name.clone(), self.element_types[ix]))
        }
        elements_verbose
    }

    pub fn get_rule_by_key(&self, index: ElementIndex) -> Option<&NonTerminalRules<T>> {
        self.parse_rules.rules.get(&index)
    }
    pub fn get_rule_by_element(&self, el: Element) -> Option<&NonTerminalRules<T>> {
        self.parse_rules.rules.get(&el.ix)
    }
    
    

    pub fn get_rule_by_element_verbose(&self, el_name: &str) -> Option<&NonTerminalRules<T>> {
        let el= ElementVerbose::new(String::from(el_name),ElementType::NonTerminal);
        let index = self.get_element_index(&el);
        if let Some(i) = index {
            self.get_rule_by_key(i)
        } else {
            None
        }
    }


    pub fn get_or_add_element_key(&mut self, identifier: &ElementVerbose) -> ElementIndex {
        return match self.element_verbose_map.get(identifier) {
            None => {
                let index = self.element_data.len();
                self.element_data.push(ElementData { keep_data: false, name: identifier.name.clone() });
                self.element_types.push(identifier.et);
                self.element_verbose_map.insert(identifier.clone(), index);
                index
            }
            Some(ix) => { *ix }
        }


    }
    pub fn get_element_index(&self, key: &ElementVerbose) -> Option<ElementIndex> {
        match self.element_verbose_map.get(key){
            None => {None}
            Some(x) => {Some(*x)}
        }
    }

    pub fn get_element_nt_index(&self, key: &str) -> Option<ElementIndex> {
        let el = ElementVerbose::new(String::from(key), ElementType::NonTerminal);
        match self.element_verbose_map.get(&el){
            None => {None}
            Some(x) => {Some(*x)}
        }
    }
    pub fn get_element_t_index(&self, key: &str) -> Option<ElementIndex> {
        let el = ElementVerbose::new(String::from(key), ElementType::Terminal);
        match self.element_verbose_map.get(&el){
            None => {None}
            Some(x) => {Some(*x)}
        }
    }

}



pub struct ParseRules<T> where T: VM {
    pub rules: RuleMap<T>,
    pub ignore: Option<ElementIndex>,
}
impl<T> fmt::Debug for ParseRules<T> where T: VM {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParseRules")
            .field("rules", &self.rules)
            .field("ignore", &self.ignore)
            .finish()
    }
}

pub type RuleMap<T> = HashMap<ElementIndex, NonTerminalRules<T>>;


pub type Ppp<'pp> = ParseProcess<'pp, PeekableWrapper<Chars<'pp>>>;


pub struct NonTerminalRules<T> where T: VM {
    pub possible_productions: PossibleProductions,
    pub ignore: Option<ElementIndex>,
    pub instruction: Option<Box<Instruction<T::Tstate>>>,
}
impl<T> fmt::Debug for NonTerminalRules<T> where T: VM {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NonTerminalRules")
            .field("possible_productions", &self.possible_productions)
            .field("ignore", &self.ignore)
            .finish()
    }
}

pub type PossibleProductions = Vec<Rc<Production>>;

pub type ElementIndex = usize;

#[derive(PartialEq, Eq, Debug, Clone,Hash)]
pub struct ElementVerbose {
    pub name: String,
    pub et: ElementType
}


impl ElementVerbose {
    pub fn new(name:String, et: ElementType) ->ElementVerbose{
        ElementVerbose{  name, et }
    }
    pub fn new_t(name:String)->ElementVerbose{
        ElementVerbose::new(name, ElementType::Terminal)
    }
    pub fn new_nt(name:String)->ElementVerbose{
        ElementVerbose::new(name, ElementType::NonTerminal)
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy,Hash)]
pub enum ElementType{
    Terminal,
    NonTerminal,
}
#[derive(PartialEq, Eq, Debug, Clone, Copy,Hash)]
pub struct Element{
    pub ix: ElementIndex,
    pub et: ElementType
}

impl Element {
    pub fn new(ix:ElementIndex,et: ElementType)->Element{
        Element{ ix, et }
    }
    pub fn new_t(ix:ElementIndex)->Element{
        Element::new(ix, ElementType::Terminal)
    }
    pub fn new_nt(ix:ElementIndex)->Element{
        Element::new(ix, ElementType::NonTerminal)
    }
}




pub struct ElementData {
    pub keep_data: bool,
    pub name:String
}
#[derive(Debug)]
pub enum Production {
    NotEmpty(Vec<ElementIndex>),
    Empty,
}