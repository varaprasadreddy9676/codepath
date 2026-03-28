package com.codepath;

import com.github.javaparser.StaticJavaParser;
import com.github.javaparser.ast.CompilationUnit;
import com.github.javaparser.ast.body.MethodDeclaration;

import java.io.File;
import java.util.ArrayList;
import java.util.List;

public class JavaASTExtractor {
    public static void main(String[] args) {
        if (args.length == 0) {
            System.err.println("Usage: java JavaASTExtractor <source_file_path>");
            System.exit(1);
        }

        try {
            File sourceFile = new File(args[0]);
            CompilationUnit cu = StaticJavaParser.parse(sourceFile);
            
            // Extract standard method structures from the AST completely decoupled from regex
            List<String> methods = new ArrayList<>();
            cu.findAll(MethodDeclaration.class).forEach(md -> {
                methods.add(md.getNameAsString());
            });

            // The exact structured return format the Rust backend expects to ingest
            System.out.println("{ \"status\": \"success\", \"methods\": [");
            for (int i=0; i < methods.size(); i++) {
                System.out.print("\"" + methods.get(i) + "\"");
                if (i < methods.size() - 1) System.out.println(",");
            }
            System.out.println("] }");
            
        } catch (Exception e) {
            System.err.println("{ \"status\": \"error\", \"message\": \"" + e.getMessage().replace("\"", "\\\"") + "\" }");
            System.exit(1);
        }
    }
}
