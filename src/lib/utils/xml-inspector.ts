export type XmlAttribute = {
	name: string;
	value: string;
};

export type XmlOffsetRange = {
	startOffset: number;
	endOffset: number;
};

export type XmlTreeItem = {
	id: string;
	name: string;
	path: string;
	attributes: XmlAttribute[];
	children: XmlTreeItem[];
	textContent: string;
	textPreview: string;
	range: XmlOffsetRange;
};

export type XmlInspectionResult = {
	formatted: string;
	root: XmlTreeItem | null;
	error: string | null;
};

type Writer = {
	readonly length: number;
	append: (value: string) => void;
	toString: () => string;
};

function createWriter(): Writer {
	const chunks: string[] = [];
	let length = 0;

	return {
		get length() {
			return length;
		},
		append(value: string) {
			chunks.push(value);
			length += value.length;
		},
		toString() {
			return chunks.join('');
		}
	};
}

function escapeText(value: string): string {
	return value.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;');
}

function escapeAttribute(value: string): string {
	return escapeText(value).replaceAll('"', '&quot;');
}

function getAttributes(element: Element): XmlAttribute[] {
	return Array.from(element.attributes).map((attribute) => ({
		name: attribute.name,
		value: attribute.value
	}));
}

function formatAttributes(element: Element): string {
	return getAttributes(element)
		.map((attribute) => ` ${attribute.name}="${escapeAttribute(attribute.value)}"`)
		.join('');
}

function getRenderableChildren(element: Element): ChildNode[] {
	return Array.from(element.childNodes).filter((child) => {
		if (child.nodeType !== Node.TEXT_NODE) {
			return true;
		}

		return (child.textContent ?? '').trim().length > 0;
	});
}

function getDirectTextContent(element: Element): string {
	return Array.from(element.childNodes)
		.filter(
			(child) => child.nodeType === Node.TEXT_NODE || child.nodeType === Node.CDATA_SECTION_NODE
		)
		.map((child) => child.textContent ?? '')
		.join('');
}

function toPreview(value: string, maxLength = 120): string {
	const normalized = value.replace(/\s+/g, ' ').trim();

	if (normalized.length <= maxLength) {
		return normalized;
	}

	return `${normalized.slice(0, maxLength - 1)}…`;
}

function serializeInlineNode(node: ChildNode): string {
	switch (node.nodeType) {
		case Node.TEXT_NODE:
			return escapeText(node.textContent ?? '');

		case Node.CDATA_SECTION_NODE:
			return `<![CDATA[${node.textContent ?? ''}]]>`;

		default:
			return '';
	}
}

function serializeElement(
	element: Element,
	depth: number,
	writer: Writer,
	nextId: () => string,
	indentText: string,
	path: string
): XmlTreeItem {
	const id = nextId();
	const attributes = getAttributes(element);
	const renderableChildren = getRenderableChildren(element);
	const textContent = getDirectTextContent(element);
	const textPreview = toPreview(textContent);
	const pad = indentText.repeat(depth);

	writer.append(pad);
	const startOffset = writer.length;

	const openTag = `<${element.tagName}${formatAttributes(element)}`;

	if (renderableChildren.length === 0) {
		writer.append(`${openTag} />`);
		const endOffset = writer.length;
		writer.append('\n');

		return {
			id,
			name: element.tagName,
			path,
			attributes,
			children: [],
			textContent,
			textPreview,
			range: { startOffset, endOffset }
		};
	}

	const inlineOnly = renderableChildren.every(
		(child) => child.nodeType === Node.TEXT_NODE || child.nodeType === Node.CDATA_SECTION_NODE
	);

	if (inlineOnly) {
		writer.append(`${openTag}>`);

		for (const child of renderableChildren) {
			writer.append(serializeInlineNode(child));
		}

		writer.append(`</${element.tagName}>`);
		const endOffset = writer.length;
		writer.append('\n');

		return {
			id,
			name: element.tagName,
			path,
			attributes,
			children: [],
			textContent,
			textPreview,
			range: { startOffset, endOffset }
		};
	}

	writer.append(`${openTag}>`);
	writer.append('\n');

	const children: XmlTreeItem[] = [];
	const siblingNameCounts: Record<string, number> = {};

	for (const child of renderableChildren) {
		if (child.nodeType === Node.ELEMENT_NODE) {
			const childElement = child as Element;
			const nextSiblingIndex = (siblingNameCounts[childElement.tagName] ?? 0) + 1;
			siblingNameCounts[childElement.tagName] = nextSiblingIndex;

			const childPath = `${path}/${childElement.tagName}[${nextSiblingIndex}]`;

			children.push(
				serializeElement(childElement, depth + 1, writer, nextId, indentText, childPath)
			);

			continue;
		}

		if (child.nodeType === Node.CDATA_SECTION_NODE) {
			writer.append(indentText.repeat(depth + 1));
			writer.append(`<![CDATA[${child.textContent ?? ''}]]>`);
			writer.append('\n');
			continue;
		}

		if (child.nodeType === Node.COMMENT_NODE) {
			writer.append(indentText.repeat(depth + 1));
			writer.append(`<!--${child.textContent ?? ''}-->`);
			writer.append('\n');
			continue;
		}

		if (child.nodeType === Node.TEXT_NODE) {
			const rawText = child.textContent ?? '';

			if (!rawText.trim()) {
				continue;
			}

			const normalizedLines = rawText
				.replace(/\r\n?/g, '\n')
				.trim()
				.split('\n')
				.map((line) => line.trim())
				.filter(Boolean);

			for (const line of normalizedLines) {
				writer.append(indentText.repeat(depth + 1));
				writer.append(escapeText(line));
				writer.append('\n');
			}
		}
	}

	writer.append(pad);
	writer.append(`</${element.tagName}>`);
	const endOffset = writer.length;
	writer.append('\n');

	return {
		id,
		name: element.tagName,
		path,
		attributes,
		children,
		textContent,
		textPreview,
		range: { startOffset, endOffset }
	};
}

export function inspectXml(xml: string, options?: { indent?: string }): XmlInspectionResult {
	const indentText = options?.indent ?? '  ';

	if (!xml.trim()) {
		return {
			formatted: '',
			root: null,
			error: null
		};
	}

	try {
		const documentNode = new DOMParser().parseFromString(xml, 'application/xml');
		const parserError =
			documentNode.querySelector('parsererror') ??
			documentNode.getElementsByTagName('parsererror')[0];

		if (parserError || !documentNode.documentElement) {
			return {
				formatted: xml,
				root: null,
				error: 'Failed to parse XML.'
			};
		}

		const writer = createWriter();
		let counter = 0;

		const nextId = (): string => {
			counter += 1;
			return `xml-node-${counter}`;
		};

		let root: XmlTreeItem | null = null;

		for (const child of Array.from(documentNode.childNodes)) {
			if (child.nodeType === Node.PROCESSING_INSTRUCTION_NODE) {
				const instruction = child as ProcessingInstruction;
				writer.append(`<?${instruction.target} ${instruction.data}?>\n`);
				continue;
			}

			if (child.nodeType === Node.DOCUMENT_TYPE_NODE) {
				const doctype = child as DocumentType;
				writer.append(`<!DOCTYPE ${doctype.name}>\n`);
				continue;
			}

			if (child.nodeType === Node.ELEMENT_NODE) {
				const rootElement = child as Element;
				root = serializeElement(
					rootElement,
					0,
					writer,
					nextId,
					indentText,
					`/${rootElement.tagName}[1]`
				);
			}
		}

		return {
			formatted: writer.toString().trimEnd(),
			root,
			error: null
		};
	} catch {
		return {
			formatted: xml,
			root: null,
			error: 'Failed to parse XML.'
		};
	}
}

export function findNodeById(node: XmlTreeItem | null, id: string): XmlTreeItem | null {
	if (!node) {
		return null;
	}

	if (node.id === id) {
		return node;
	}

	for (const child of node.children) {
		const found = findNodeById(child, id);

		if (found) {
			return found;
		}
	}

	return null;
}

export function findDeepestNodeAtOffset(
	node: XmlTreeItem | null,
	offset: number
): XmlTreeItem | null {
	if (!node) {
		return null;
	}

	if (offset < node.range.startOffset || offset >= node.range.endOffset) {
		return null;
	}

	for (const child of node.children) {
		const found = findDeepestNodeAtOffset(child, offset);

		if (found) {
			return found;
		}
	}

	return node;
}

export function getPathToNodeIds(
	node: XmlTreeItem | null,
	id: string,
	trail: string[] = []
): string[] {
	if (!node) {
		return [];
	}

	const nextTrail = [...trail, node.id];

	if (node.id === id) {
		return nextTrail;
	}

	for (const child of node.children) {
		const result = getPathToNodeIds(child, id, nextTrail);

		if (result.length > 0) {
			return result;
		}
	}

	return [];
}

export function collectNodeIds(node: XmlTreeItem | null): string[] {
	if (!node) {
		return [];
	}

	const ids = [node.id];

	for (const child of node.children) {
		ids.push(...collectNodeIds(child));
	}

	return ids;
}

export function countNodes(node: XmlTreeItem | null): number {
	if (!node) {
		return 0;
	}

	let total = 1;

	for (const child of node.children) {
		total += countNodes(child);
	}

	return total;
}

export function nodeMatchesQuery(node: XmlTreeItem, query: string): boolean {
	const needle = query.trim().toLowerCase();

	if (!needle) {
		return true;
	}

	if (node.name.toLowerCase().includes(needle)) {
		return true;
	}

	if (node.path.toLowerCase().includes(needle)) {
		return true;
	}

	if (node.textPreview.toLowerCase().includes(needle)) {
		return true;
	}

	return node.attributes.some(
		(attribute) =>
			attribute.name.toLowerCase().includes(needle) ||
			attribute.value.toLowerCase().includes(needle)
	);
}

export function filterXmlTree(node: XmlTreeItem | null, query: string): XmlTreeItem | null {
	if (!node) {
		return null;
	}

	const needle = query.trim();

	if (!needle) {
		return node;
	}

	const filteredChildren = node.children
		.map((child) => filterXmlTree(child, needle))
		.filter((child): child is XmlTreeItem => child !== null);

	if (nodeMatchesQuery(node, needle) || filteredChildren.length > 0) {
		return {
			...node,
			children: filteredChildren
		};
	}

	return null;
}

export function getLineAndColumnFromOffset(
	text: string,
	offset: number
): { line: number; column: number } {
	const safeOffset = Math.max(0, Math.min(offset, text.length));

	let line = 1;
	let column = 1;

	for (let index = 0; index < safeOffset; index += 1) {
		if (text[index] === '\n') {
			line += 1;
			column = 1;
		} else {
			column += 1;
		}
	}

	return { line, column };
}
