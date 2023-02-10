/*
    structure permmettant de conserver l'origine de chaque instruction afin de permettre de lever les erreurs correctement
*/
struct Context {
    name: String,
}

enum ContextType {
    FunctionDeclaration(String),
    Goto,
}
