// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="introduction.html">Introduction</a></li><li class="chapter-item expanded affix "><li class="part-title">Getting Started</li><li class="chapter-item expanded "><a href="installation.html"><strong aria-hidden="true">1.</strong> Installation</a></li><li class="chapter-item expanded "><a href="quick-start.html"><strong aria-hidden="true">2.</strong> Quick Start</a></li><li class="chapter-item expanded "><a href="first-transpilation.html"><strong aria-hidden="true">3.</strong> Your First Transpilation</a></li><li class="chapter-item expanded affix "><li class="part-title">Core Concepts</li><li class="chapter-item expanded "><a href="how-it-works.html"><strong aria-hidden="true">4.</strong> How Decy Works</a></li><li class="chapter-item expanded "><a href="pipeline.html"><strong aria-hidden="true">5.</strong> The Transpilation Pipeline</a><a class="toggle"><div>❱</div></a></li><li><ol class="section"><li class="chapter-item "><a href="pipeline/parser.html"><strong aria-hidden="true">5.1.</strong> Parser (C AST)</a></li><li class="chapter-item "><a href="pipeline/hir.html"><strong aria-hidden="true">5.2.</strong> HIR (High-level IR)</a></li><li class="chapter-item "><a href="pipeline/ownership.html"><strong aria-hidden="true">5.3.</strong> Ownership Inference</a></li><li class="chapter-item "><a href="pipeline/codegen.html"><strong aria-hidden="true">5.4.</strong> Code Generation</a></li></ol></li><li class="chapter-item expanded "><a href="ownership-safety.html"><strong aria-hidden="true">6.</strong> Ownership &amp; Safety</a></li><li class="chapter-item expanded affix "><li class="part-title">C-to-Rust Patterns</li><li class="chapter-item expanded "><a href="patterns/pointers.html"><strong aria-hidden="true">7.</strong> Pointers to References</a></li><li class="chapter-item expanded "><a href="patterns/arrays.html"><strong aria-hidden="true">8.</strong> Arrays and Slices</a></li><li class="chapter-item expanded "><a href="patterns/structs.html"><strong aria-hidden="true">9.</strong> Structs and Enums</a></li><li class="chapter-item expanded "><a href="patterns/functions.html"><strong aria-hidden="true">10.</strong> Functions</a></li><li class="chapter-item expanded "><a href="patterns/control-flow.html"><strong aria-hidden="true">11.</strong> Control Flow</a></li><li class="chapter-item expanded "><a href="patterns/memory.html"><strong aria-hidden="true">12.</strong> Memory Management</a></li><li class="chapter-item expanded "><a href="patterns/string-safety.html"><strong aria-hidden="true">13.</strong> String Safety</a></li><li class="chapter-item expanded "><a href="patterns/loop-array-safety.html"><strong aria-hidden="true">14.</strong> Loop + Array Safety</a></li><li class="chapter-item expanded "><a href="patterns/dynamic-memory-safety.html"><strong aria-hidden="true">15.</strong> Dynamic Memory Safety</a></li><li class="chapter-item expanded "><a href="patterns/pointer-arithmetic-safety.html"><strong aria-hidden="true">16.</strong> Pointer Arithmetic Safety</a></li><li class="chapter-item expanded "><a href="patterns/type-casting-safety.html"><strong aria-hidden="true">17.</strong> Type Casting Safety</a></li><li class="chapter-item expanded "><a href="patterns/null-pointer-safety.html"><strong aria-hidden="true">18.</strong> NULL Pointer Safety</a></li><li class="chapter-item expanded "><a href="patterns/integer-overflow-safety.html"><strong aria-hidden="true">19.</strong> Integer Overflow Safety</a></li><li class="chapter-item expanded "><a href="patterns/buffer-overflow-safety.html"><strong aria-hidden="true">20.</strong> Buffer Overflow Safety</a></li><li class="chapter-item expanded "><a href="patterns/use-after-free-safety.html"><strong aria-hidden="true">21.</strong> Use-After-Free Safety</a></li><li class="chapter-item expanded "><a href="patterns/uninitialized-memory-safety.html"><strong aria-hidden="true">22.</strong> Uninitialized Memory Safety</a></li><li class="chapter-item expanded "><a href="patterns/format-string-safety.html"><strong aria-hidden="true">23.</strong> Format String Safety</a></li><li class="chapter-item expanded "><a href="patterns/race-condition-safety.html"><strong aria-hidden="true">24.</strong> Race Condition Safety</a></li><li class="chapter-item expanded "><a href="patterns/double-free-safety.html"><strong aria-hidden="true">25.</strong> Double Free Safety</a></li><li class="chapter-item expanded affix "><li class="part-title">Advanced Topics</li><li class="chapter-item expanded "><a href="advanced/multi-file.html"><strong aria-hidden="true">26.</strong> Multi-file Projects</a></li><li class="chapter-item expanded "><a href="advanced/migration.html"><strong aria-hidden="true">27.</strong> Incremental Migration</a></li><li class="chapter-item expanded "><a href="advanced/ffi.html"><strong aria-hidden="true">28.</strong> FFI Boundaries</a></li><li class="chapter-item expanded "><a href="advanced/cache.html"><strong aria-hidden="true">29.</strong> Cache System</a></li><li class="chapter-item expanded "><a href="advanced/ml-features.html"><strong aria-hidden="true">30.</strong> ML-Enhanced Ownership</a></li><li class="chapter-item expanded "><a href="advanced/oracle.html"><strong aria-hidden="true">31.</strong> Oracle Integration (CITL)</a></li><li class="chapter-item expanded "><a href="advanced/debugging.html"><strong aria-hidden="true">32.</strong> Debugging</a></li><li class="chapter-item expanded affix "><li class="part-title">Reference</li><li class="chapter-item expanded "><a href="reference/cli.html"><strong aria-hidden="true">33.</strong> CLI Reference</a></li><li class="chapter-item expanded "><a href="reference/config.html"><strong aria-hidden="true">34.</strong> Configuration</a></li><li class="chapter-item expanded "><a href="reference/limitations.html"><strong aria-hidden="true">35.</strong> Known Limitations</a></li><li class="chapter-item expanded "><a href="reference/troubleshooting.html"><strong aria-hidden="true">36.</strong> Troubleshooting</a></li><li class="chapter-item expanded affix "><li class="part-title">Development</li><li class="chapter-item expanded "><a href="development/contributing.html"><strong aria-hidden="true">37.</strong> Contributing</a></li><li class="chapter-item expanded "><a href="development/architecture.html"><strong aria-hidden="true">38.</strong> Architecture</a></li><li class="chapter-item expanded "><a href="development/testing.html"><strong aria-hidden="true">39.</strong> Testing</a></li><li class="chapter-item expanded "><a href="development/releases.html"><strong aria-hidden="true">40.</strong> Release Process</a></li><li class="chapter-item expanded affix "><li class="spacer"></li><li class="chapter-item expanded affix "><a href="appendix-c99.html">Appendix: C99 Validation</a></li><li class="chapter-item expanded affix "><a href="appendix-unsafe.html">Appendix: Unsafe Minimization</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString();
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
